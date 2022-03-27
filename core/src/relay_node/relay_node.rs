use std::{borrow::BorrowMut, sync::Arc};

use async_std::{
    io::{Error, ErrorKind, Result, Write, Cursor},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    prelude::*,
    sync::{Mutex},
    task,
};

use crate::crypto::Aes256;
use crate::{
    protocol::{
        io::{RawOnionReader, RawOnionWriter},
        onion::{Message, Onion, Target, HelloRequest, ClientType, Relay},
    }, crypto::{ClientCrypto, ServerSecret, ClientSecret, ServerCrypto},
};

use super::{tunnel::Tunnel, relay_context::{RelayContext, Circuit}};

pub struct RelayNode {
    ip: IpAddr,
    port: u16,
    context: Arc<Mutex<RelayContext>>,
}

impl RelayNode {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip: ip,
            port: port,
            context: Arc::new(Mutex::new(RelayContext::new())),
        }
    }

    pub fn start(&self) {
        let socket = SocketAddr::new(self.ip, self.port);
        let listen_future = self.listen(socket);

        task::block_on(listen_future); // bytte til async?
    }

    pub fn register(&self, index_addr: SocketAddr, pub_key: [u8; 32], index_signing_pub_key: [u8; 32]) {
        let register_future = async {
            let tunnel = Self::index_tunnel(index_addr, pub_key, index_signing_pub_key).await.expect("Failed to establish onion tunnel with Index node");

            tunnel.send_onion(
                Onion {
                    target: Target::Current,
                    circuit_id: None,
                    message: Message::RelayPingRequest()
                }
            ).await
            .expect("Failed to send ping request to index node");

            let response = tunnel.recv_onion().await;
        };

        task::block_on(register_future);
    }

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
    
    async fn handle_connection(
        stream: TcpStream,
        context: Arc<Mutex<RelayContext>>,
    ) -> Result<()> {
        let (circuit_id, peel_tunnel_arc, hello_req) = {
            let mut guard = context.lock().await;
            let context_locked = &mut *guard;
    
            let secret = context_locked.crypto.gen_secret();
            let pub_key = secret.public_key();
    
            let (peel_tunnel, hello_req) = Self::establish_sender_tunnel(stream, secret).await?;
            let peel_tunnel_arc = Arc::new(peel_tunnel);
    
            if hello_req.client_type == ClientType::Relay {
                context_locked.relay_tunnels.insert(peel_tunnel_arc.peer_addr(), peel_tunnel_arc.clone());
            }
    
            peel_tunnel_arc.send_onion(
                Onion {
                    target: Target::Current,
                    circuit_id: None,
                    message: Message::HelloResponse(pub_key),
                }
            ).await?;

            (
                context_locked.circ_id_generator.get_uid(),
                peel_tunnel_arc.clone(),
                hello_req
            )
        };
        

        // while new onion decoded from tunnel
        // 

        // Relay recv loop
        while let onion = peel_tunnel_arc.recv_onion().await {
            let mut guard = context.lock().await;
            let context_locked = &mut *guard;

            if onion.target != Target::Current {
                panic!("onion target was not current");
            }

            match hello_req.client_type {
                ClientType::Consumer => { //means we are relay_1
                    let relay_id = match onion.target {
                        Target::Relay(id) => id,
                        _ => panic!("must have a relay to send to!")
                    };
                    let layer_tunnel_arc = Self::relay_tunnel(relay_id, context.clone()).await?;

                    let onion = Onion {
                        target: Target::Current,
                        circuit_id: Some(circuit_id),
                        message: onion.message
                    };

                    layer_tunnel_arc.send_onion(onion).await?;
                },
                ClientType::Relay => {
                    let prev_circuit_id = onion.circuit_id.expect("did not receive circuit id");

                    match onion.message {
                        Message::HelloRequest(req) => {
                            let new_circuit_id = context_locked.circ_id_generator.get_uid();
                            let relay_id = match onion.target {
                                Target::Relay(id) => id,
                                _ => panic!("must have a relay to send to!")
                            };
                            let layer_tunnel_arc = Self::relay_tunnel(relay_id, context.clone()).await?;

                            let symmetric_cipher = context_locked.crypto.gen_secret().symmetric_cipher(req.public_key);

                            context_locked.circuits.insert(prev_circuit_id, Arc::new(
                                Circuit {
                                    id: new_circuit_id,
                                    symmetric_cipher,
                                    peel_tunnel_addr: peel_tunnel_arc.peer_addr(),
                                    layer_tunnel_addr: layer_tunnel_arc.peer_addr(),
                                    endpoint_connection: None
                                }
                            ));
                        },
                        Message::Payload(ref payload) => {
                            let circuit = context_locked.circuits.get(&prev_circuit_id).map(|circ_ref| circ_ref.clone()).expect("ewrw");
                    
                            let peeled_onion = peel_tunnel_arc.peel_layer(payload.to_vec(), circuit.symmetric_cipher.clone()).await?;

                            match peeled_onion.target {
                                Target::Current => {
                                    panic!("got current")
                                },
                                Target::Relay(id) => {
                                    drop(guard);
                                    let layer_tunnel_arc = Self::relay_tunnel(id, context.clone()).await?;

                                    let onion = Onion {
                                        target: Target::Current,
                                        circuit_id: Some(circuit.id),
                                        message: peeled_onion.message
                                    };

                                    layer_tunnel_arc.send_onion(onion).await?;
                                },
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
                        },
                        _ => {
                            
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn relay_ip(stream: TcpStream) {
        // tentative

    }

    async fn relay_tunnel(relay_id: u32, context: Arc<Mutex<RelayContext>>) -> Result<Arc<Tunnel>> {
        let mut guard = context.lock().await;
        let context_locked = &mut *guard;

        let relay
            = context_locked.indexed_relays
                .iter()
                .find(|relay| relay.id == relay_id)
                .expect("Relay not indexed");

            let tunnel = if let Some(existing_tunnel)
                = context_locked.relay_tunnels
                    .contains_key(&relay.addr)
                    .then(|| context_locked.relay_tunnels.get(&relay.addr))
                    .map(|tunnel| tunnel.unwrap())
            {
                existing_tunnel.clone()
            } else {
                let crypto = ClientCrypto::new(&relay.pub_key).expect("Failed to generate crypto");
                let secret = crypto.gen_secret();

                let tunnel = Arc::new(Tunnel::reach_relay(TcpStream::connect(relay.addr).await?, secret).await?);
                context_locked.relay_tunnels.insert(relay.addr, tunnel.clone());

                tunnel.clone() 
            };

            Ok(tunnel)
    }

    async fn establish_sender_tunnel(stream: TcpStream, secret: ServerSecret) -> Result<(Tunnel, HelloRequest)> {
        let reader = &mut RawOnionReader::new(&stream);

        let sender_hello = reader.read().await?;
        if let Message::HelloRequest(req) = sender_hello.message {
            Ok((Tunnel::new(stream, secret.symmetric_cipher(req.public_key)), req))
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Expected Hello request"))
        }
    }

    async fn establish_receiver_channel(stream: TcpStream, secret: ClientSecret) {

    }

    async fn index_all_relays(index_addr: SocketAddr, pub_key: [u8; 32], index_signing_pub_key: [u8; 32]) -> Result<Vec<Relay>> {
        let tunnel = Self::index_tunnel(index_addr, pub_key, index_signing_pub_key).await?;

        tunnel.send_onion(
            Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::GetRelaysRequest()
        }).await?;

        let relays_response = tunnel.recv_onion().await;
        if let Message::GetRelaysResponse(relays) = relays_response.message {
            Ok(relays)
        } else {
            //err?
            todo!()
        }
    }

    async fn index_tunnel(addr: SocketAddr, pub_key: [u8; 32], index_signing_pub_key: [u8; 32]) -> Result<Tunnel> {
        let stream = TcpStream::connect(addr).await?;
        
        let mut writer = RawOnionWriter::new(&stream);
        writer.write(
            Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::HelloRequest(HelloRequest { client_type: ClientType::Relay, public_key: pub_key })
            }).await?;
            
        let mut reader = RawOnionReader::new(&stream);
        let hello_response = reader.read().await?;
        
        let symmetric_cipher = if let Message::HelloResponse(peer_key) = hello_response.message {
            let crypto = ClientCrypto::new(&index_signing_pub_key).expect("Failed to generate crypto");
            crypto.gen_secret().symmetric_cipher(peer_key).expect("Failed to generate symmetric cipher")
        } else {
            //err?
            todo!()
        };

        Ok(Tunnel::new(stream, symmetric_cipher))
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