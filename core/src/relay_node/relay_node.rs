use std::{borrow::BorrowMut, sync::Arc};

use async_std::{
    io::{Cursor, Error, ErrorKind, Result, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    prelude::*,
    sync::Mutex,
    task,
};

use crate::{crypto::Aes256, protocol::onion::RelayPingRequest};
use crate::{
    crypto::{ClientCrypto, ClientSecret, ServerCrypto, ServerSecret},
    protocol::{
        io::{RawOnionReader, RawOnionWriter},
        onion::{ClientType, HelloRequest, Message, Onion, Relay, Target},
    },
};

use super::{
    relay_context::{Circuit, RelayContext},
    tunnel::OnionTunnel,
};

pub struct RelayNode {
    ip: IpAddr,
    port: u16,
    context: Arc<Mutex<RelayContext>>,
}

impl RelayNode {
    // Returns a new RelayNode object
    // param ip: The IP address this relay node should bind to
    // param port: The port this relay node should listen on
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip,
            port,
            context: Arc::new(Mutex::new(RelayContext::new())),
        }
    }

    // Starts the RelayNode server, causing it to listen to the socket address specified in RelayNode::new()
    pub fn start(&self) {
        let socket = SocketAddr::new(self.ip, self.port);
        let listen_future = self.listen(socket);

        task::block_on(listen_future); // bytte til async?
    }

    // Registers the relay node at the specified index node, making it visible to other relay nodes and consumers
    // param index_addr: The socket address of the index node
    // param index_signing_pub_key: The signing public key of the index node
    pub fn register(&self, index_addr: SocketAddr, index_signing_pub_key: [u8; 32]) {
        let register_future = async {
            let locked_context = self.context.lock().await;

            let tunnel = Self::index_tunnel(index_addr, index_signing_pub_key)
                .await
                .expect("Failed to establish onion tunnel with Index node");

            tunnel
                .send_onion(Onion {
                    target: Target::Current,
                    circuit_id: None,
                    message: Message::RelayPingRequest(RelayPingRequest {
                        port: self.port,
                        signing_public: locked_context.crypto.signing_public(),
                    }),
                })
                .await
                .expect("Failed to send ping request to index node");

            let _ = tunnel.recv_onion().await;
        };

        task::block_on(register_future);

        let index_relays_future = async {
            self.context.lock().await.indexed_relays.extend(
                Self::index_all_relays(index_addr, index_signing_pub_key)
                    .await
                    .expect("msg"),
            )
        };

        task::block_on(index_relays_future);
    }

    // Helper method for listening on a socket address and handling the incoming connections
    // param socket: The specified socket address to listen on
    async fn listen(&self, socket: SocketAddr) {
        let listener = TcpListener::bind(socket)
            .await
            .expect("Failed to bind to socket");

        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let context = self.context.clone();
            let handler_future = async {
                Self::handle_connection(stream.expect("Failed to read from stream"), context)
                    .await
                    .expect("Failed to handle connection")
            };

            task::spawn(handler_future);
        }
    }

    // Helper method for handling a TcpStream connection and respond to index node queries
    // param stream: The TCP stream used in the connection to handle
    // param context: Index node context required for management of circuits, tunnels, id generation and cryptography in a static context
    async fn handle_connection(stream: TcpStream, context: Arc<Mutex<RelayContext>>) -> Result<()> {
        let (circuit_id, peel_tunnel_arc, hello_req) = {
            let mut guard = context.lock().await;
            let context_locked = &mut *guard;

            let secret = context_locked.crypto.gen_secret();
            let pub_key = secret.public_key();

            let (peel_tunnel, hello_req) =
                Self::establish_sender_tunnel(stream.clone(), secret).await?;
            let peel_tunnel_arc = Arc::new(peel_tunnel);

            if hello_req.client_type == ClientType::Relay {
                context_locked
                    .relay_tunnels
                    .insert(peel_tunnel_arc.peer_addr(), peel_tunnel_arc.clone());
            }

            // TODO: refactor to not use cloned stream
            RawOnionWriter::new(stream)
                .write(Onion {
                    target: Target::Current,
                    circuit_id: None,
                    message: Message::HelloResponse(pub_key),
                })
                .await?;

            (
                context_locked.circ_id_generator.get_uid(),
                peel_tunnel_arc.clone(),
                hello_req,
            )
        };

        // while new onion decoded from tunnel
        //

        // Relay recv loop
        while let onion = peel_tunnel_arc.recv_onion().await {
            let mut guard = context.lock().await;
            let context_locked = &mut *guard;

            match hello_req.client_type {
                ClientType::Consumer => {
                    println!("Got consumer contact");
                    //means we are relay_1
                    let relay_id = match onion.target {
                        Target::Relay(id) => id,
                        _ => panic!("must have a relay to send to!"),
                    };

                    drop(guard);
                    let layer_tunnel_arc = Self::relay_tunnel(relay_id, context.clone()).await?;

                    let onion = Onion {
                        target: Target::Current,
                        circuit_id: Some(circuit_id),
                        message: onion.message,
                    };
                    println!("New Onion: {:?}", onion);

                    layer_tunnel_arc.send_onion(onion).await?;
                }

                //relay_1 sends onion {
                //  target: current,
                //  circuit_id: Some(325555555),
                //  message: HelloRequest(data)
                //} to_relay_2
                ClientType::Relay => {
                    println!("Got relay contact");
                    let prev_circuit_id = onion.circuit_id.expect("did not receive circuit id");

                    match onion.message {
                        Message::HelloRequest(req) => {
                            let new_circuit_id = context_locked.circ_id_generator.get_uid();
                            match onion.target {
                                Target::Relay(id) => {
                                    let layer_tunnel_arc =
                                        Self::relay_tunnel(id, context.clone()).await?;

                                    let symmetric_cipher = context_locked
                                        .crypto
                                        .gen_secret()
                                        .symmetric_cipher(req.public_key);

                                    context_locked.circuits.insert(
                                        prev_circuit_id,
                                        Arc::new(Circuit {
                                            id: new_circuit_id,
                                            symmetric_cipher,
                                            peel_tunnel_addr: peel_tunnel_arc.peer_addr(),
                                            layer_tunnel_addr: layer_tunnel_arc.peer_addr(),
                                            endpoint_connection: None,
                                        }),
                                    );
                                    println!("got money");
                                }
                                Target::Current => {
                                    let pub_key = context_locked.crypto.gen_secret().public_key();

                                    peel_tunnel_arc
                                        .send_onion(Onion {
                                            target: Target::Current,
                                            circuit_id: None,
                                            message: Message::HelloResponse(pub_key),
                                        })
                                        .await?;
                                    println!("fucked bitches");
                                }
                                _ => panic!("must have a relay to send to!"),
                            };
                        }
                        Message::Payload(ref payload) => {
                            let circuit = context_locked
                                .circuits
                                .get(&prev_circuit_id)
                                .map(|circ_ref| circ_ref.clone())
                                .expect("ewrw");

                            let peeled_onion = peel_tunnel_arc
                                .peel_layer(payload.to_vec(), circuit.symmetric_cipher.clone())
                                .await?;

                            match peeled_onion.target {
                                Target::Current => {
                                    panic!("got current")
                                }
                                Target::Relay(id) => {
                                    drop(guard);
                                    let layer_tunnel_arc =
                                        Self::relay_tunnel(id, context.clone()).await?;

                                    let onion = Onion {
                                        target: Target::Current,
                                        circuit_id: Some(circuit.id),
                                        message: peeled_onion.message,
                                    };

                                    layer_tunnel_arc.send_onion(onion).await?;
                                }
                                Target::IP(ip) => {
                                    //check if connected
                                    //false -> connect
                                    //true -> send to the conn conn
                                    //send Message::Payload(data) => data

                                    // let circuit = context_locked.circuits.get(&circuit_id).map(|circ| circ.clone()).expect("failed to get circuit");

                                    // if let Some(connection) = circuit.endpoint_connection {
                                    //     let (reader, writer) = &mut (&connection, &connection);

                                    //     match onion.message {
                                    //         Message::Payload(payload) => {

                                    //         },
                                    //         _ => todo!()
                                    //     }
                                    // } else {
                                    //     // connect
                                    //     circuit.endpoint_connection = Some(TcpStream::connect(ip).await?);
                                    // }
                                    let data = match onion.message {
                                        Message::Payload(data) => data,
                                        _ => panic!("Wrong message type"),
                                    };

                                    println!("Received data: {:?}", data)
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    //
    async fn relay_ip(stream: TcpStream) {
        // tentative
    }

    // Gets or creates a new secure onion tunnel to another relay based on whether or not there exists a previous connection to said relay
    // param relay_id: The public id of the relay to connect to, used by the index node
    // param context: Index node context required for management of circuits, tunnels, id generation and cryptography in a static context
    async fn relay_tunnel(
        relay_id: u32,
        context: Arc<Mutex<RelayContext>>,
    ) -> Result<Arc<OnionTunnel>> {
        let mut guard = context.lock().await;
        let context_locked = &mut *guard;

        let relay = context_locked
            .indexed_relays
            .iter()
            .find(|relay| relay.id == relay_id)
            .expect("Relay not indexed");

        let tunnel = if let Some(existing_tunnel) = context_locked
            .relay_tunnels
            .contains_key(&relay.addr)
            .then(|| context_locked.relay_tunnels.get(&relay.addr))
            .map(|tunnel| tunnel.unwrap())
        {
            existing_tunnel.clone()
        } else {
            let crypto = ClientCrypto::new(&relay.pub_key).expect("Failed to generate crypto");
            let secret = crypto.gen_secret();

            let tunnel = Arc::new(
                OnionTunnel::reach_relay(TcpStream::connect(relay.addr).await?, secret).await?,
            );
            context_locked
                .relay_tunnels
                .insert(relay.addr, tunnel.clone());

            tunnel.clone()
        };

        Ok(tunnel)
    }

    // Establishes a secure onion tunnel on the given connection
    // param stream: Connection to establish an onion tunnel on
    // param secret: The secret to use with the onion tunnel
    async fn establish_sender_tunnel(
        stream: TcpStream,
        secret: ServerSecret,
    ) -> Result<(OnionTunnel, HelloRequest)> {
        let reader = &mut RawOnionReader::new(&stream);

        let sender_hello = reader.read().await?;
        if let Message::HelloRequest(req) = sender_hello.message {
            Ok((
                OnionTunnel::new(stream, secret.symmetric_cipher(req.public_key)),
                req,
            ))
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Expected Hello request"))
        }
    }

    async fn establish_receiver_channel(stream: TcpStream, secret: ClientSecret) {}

    // Contacts the given index node and returns all other registered indexes on the onion network
    // param index_addr: The socket address of the index node
    // param index_signing_pub_key: The signing public key of the index node
    async fn index_all_relays(
        index_addr: SocketAddr,
        index_signing_pub_key: [u8; 32],
    ) -> Result<Vec<Relay>> {
        let tunnel = Self::index_tunnel(index_addr, index_signing_pub_key).await?;

        tunnel
            .send_onion(Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::GetRelaysRequest(),
            })
            .await?;

        let relays_response = tunnel.recv_onion().await;
        if let Message::GetRelaysResponse(relays) = relays_response.message {
            Ok(relays)
        } else {
            //err?
            todo!()
        }
    }

    // Establishes a new secure onion tunnel to the given index node
    // param index_addr: The socket address of the index node
    // param index_signing_pub_key: The signing public key of the index node
    async fn index_tunnel(
        addr: SocketAddr,
        index_signing_pub_key: [u8; 32],
    ) -> Result<OnionTunnel> {
        let stream = TcpStream::connect(addr).await?;
        let crypto = ClientCrypto::new(&index_signing_pub_key).expect("Failed to generate crypto");
        let secret = crypto.gen_secret();

        let mut writer = RawOnionWriter::new(&stream);
        writer
            .write(Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::HelloRequest(HelloRequest {
                    client_type: ClientType::Relay,
                    public_key: secret.public_key(),
                }),
            })
            .await?;

        let mut reader = RawOnionReader::new(&stream);
        let hello_response = reader.read().await?;

        let symmetric_cipher = if let Message::HelloResponse(peer_key) = hello_response.message {
            secret
                .symmetric_cipher(peer_key)
                .expect("failed to generate symmetric cipher")
        } else {
            //err?
            todo!()
        };

        Ok(OnionTunnel::new(stream, symmetric_cipher))
    }
}

// match context_locked.relay_tunnels.get(&circuit.layer_tunnel_addr) {
//     Some(tunnel) => tunnel.clone(),
//     None => {
//         match onion_to_relay.target {
//             Target::Relay(relay_id) => {
//                 drop(guard);
//                 Self::relay_tunnel(relay_id, context.clone()).await?
//             },
//             Target::IP(ip) => {

//                 todo!();
//             },
//             Target::Current => todo!() // err?
//         }
//     }
// }
