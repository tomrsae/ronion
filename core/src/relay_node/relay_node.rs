use std::borrow::BorrowMut;

use aes::Aes256;
use async_std::{
    io::{Error, ErrorKind, Result, Write, Cursor},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    prelude::*,
    sync::{Arc, Mutex},
    task,
};

use crate::{
    protocol::{
        io::{RawOnionReader, RawOnionWriter},
        onion::{Message, Onion, Target, HelloRequest, ClientType, Relay},
    }, crypto::{ClientCrypto, ServerSecret, ClientSecret},
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
        let mut guard = context.lock().await;
        let context_locked = &mut *guard;

        let secret = context_locked.crypto.gen_secret();
        let pub_key = secret.public_key();

        let (sender_tunnel, connector_type) = Self::establish_sender_tunnel(stream, secret).await?;
        let sender_channel_arc = Arc::new(sender_tunnel);

        let mut circuit_id = None;
        match connector_type {
            ClientType::Consumer => {
                circuit_id = Some(context_locked.circ_id_generator.get_uid());
                context_locked
                    .circuits
                    .insert(circuit_id.unwrap(), Circuit {
                    
                    });
            }
            ClientType::Relay => {
                context_locked.tunnels.insert(sender_channel_arc.peer_addr(), sender_channel_arc.clone());
            }
        }

        sender_channel_arc.send_onion(
            Onion {
                target: Target::Current,
                circuit_id: circuit_id,
                message: Message::HelloResponse(pub_key),
            }
        ).await?;

        // Relay recv loop
        while let onion_to_relay = sender_channel_arc.recv_onion().await? {
            let mut guard = context.lock().await;
            let context_locked = &mut *guard;
    
            let receiver_channel = match onion_to_relay.target {
                Target::Relay(relay_id) => {
                    let relay
                        = context_locked.indexed_relays
                            .iter()
                            .find(|relay| relay.id == relay_id)
                            .expect("Relay not indexed");

                    if let Some(existing_tunnel)
                        = context_locked.tunnels
                            .contains_key(&relay.addr)
                            .then(|| context_locked.tunnels.get(&relay.addr))
                            .map(|tunnel| tunnel.unwrap())
                    {
                        existing_tunnel.clone()
                    } else {
                        let crypto = ClientCrypto::new(&relay.pub_key).expect("Failed to generate crypto");
                        let secret = crypto.gen_secret();
    
                        let channel = Arc::new(Tunnel::reach_relay(TcpStream::connect(relay.addr).await?, secret).await?);
                        context_locked.tunnels.insert(relay.addr, channel.clone());

                        channel
                    }
                },
                Target::IP(ip) => {
                    // I am exit node
                    // not sure bud, ask de bois

                    todo!();
                },
                Target::Current => todo!() // err?
            };

            // peel or layer onion further before relaying

            receiver_channel.send_onion(onion_to_relay).await?;
        }

        Ok(())
    }

    async fn establish_sender_tunnel(stream: TcpStream, secret: ServerSecret) -> Result<(Tunnel, ClientType)> {
        let reader = &mut RawOnionReader::new(&stream);

        let sender_hello = reader.read().await?;
        if let Message::HelloRequest(req) = sender_hello.message {
            Ok((Tunnel::new(stream, secret.symmetric_cipher(req.public_key)), req.client_type))
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Expected Hello request"))
        }
    }

    async fn establish_receiver_channel(stream: TcpStream, secret: ClientSecret) {

    }

    async fn index_all_relays(index_addr: &str, pub_key: [u8; 32], index_signing_pub_key: [u8; 32]) -> Result<Vec<Relay>> {
        let request = Onion {
            target: Target::Current,
            circuit_id: None,
            message: Message::GetRelaysRequest()
        };

        let stream = TcpStream::connect(index_addr).await?;

        let mut writer = RawOnionWriter::new(&stream);
        writer.write(
            Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::HelloRequest(HelloRequest { client_type: ClientType::Relay, public_key: pub_key })
        }).await?;

        let mut reader = RawOnionReader::new(&stream);
        let hello_response = reader.read().await?;

        let symmetric_cipher =
        if let Message::HelloResponse(peer_key) = hello_response.message {
            let crypto = ClientCrypto::new(&index_signing_pub_key).expect("Failed to generate crypto");
            crypto.gen_secret().symmetric_cipher(peer_key).expect("Failed to generate symmetric cipher")
        } else {
            //err?
            todo!()
        };
        
        writer.with_cipher(symmetric_cipher.clone()).write(
            Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::GetRelaysRequest()
        }).await?;

        let relays_response = reader.with_cipher(symmetric_cipher).read().await?;
        if let Message::GetRelaysResponse(relays) = relays_response.message {
            Ok(relays)
        } else {
            //err?
            todo!()
        }
    }
}
