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

use super::{channel::OnionChannel, relay_context::RelayContext};

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

        let (sender_channel, connector_type) = Self::establish_sender_channel(stream, secret).await?;
        let sender_channel_arc = Arc::new(sender_channel);

        let mut circuit_id = None;
        match connector_type {
            ClientType::Consumer => {
                circuit_id = Some(context_locked.circ_id_generator.get_uid());
                context_locked
                    .circuits
                    .insert(circuit_id.unwrap(), sender_channel_arc.clone());
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

        while let onion = sender_channel_arc.recv_onion().await? {
            let mut guard = context.lock().await;
            let context_locked = &mut *guard;
    
            let receiver_channel = match onion.target {
                Target::Relay(relay_id) => {
                    let relay = context_locked.indexed_relays.iter().find(|relay| relay.id == relay_id).expect("Relay not indexed");

                    let crypto = ClientCrypto::new(&relay.pub_key).expect("Failed to generate crypto");
                    let secret = crypto.gen_secret();
                    let channel = OnionChannel::reach_relay(TcpStream::connect(relay.addr).await?, secret).await?;

                    // peel or layer here
                    let onion = todo!();
    
                    channel.send_onion(onion).await?;
    
                    context_locked.tunnels.insert(relay.addr, Arc::new(channel));
                },
                Target::IP(ip) => {
                    // I am exit node
                    todo!();
                },
                Target::Current => todo!() // err?
            };
    
            // if let Message::Payload(payload) = onion.message {
            //     todo!();
            // } else {
            //     // err?
            //     todo!();
            // }
        }

        Ok(())
    }

    async fn establish_sender_channel(stream: TcpStream, secret: ServerSecret) -> Result<(OnionChannel, ClientType)> {
        let (reader, writer)
            = &mut (RawOnionReader::new(&stream), RawOnionWriter::new(&stream));

        let sender_hello = reader.read().await?;
        let hello_req = if let Message::HelloRequest(req) = sender_hello.message {
            Ok(req)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Expected Hello request"))
        }?;

        Ok((OnionChannel::new(stream, secret.symmetric_cipher(hello_req.public_key)), hello_req.client_type))
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
