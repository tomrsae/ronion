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
    }, crypto::ClientCrypto,
};

use super::{
    relay_context::RelayContext, channel::OnionChannel,
};

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
        let mut reader = RawOnionReader::new(&stream);

        let hello_onion = reader.read().await?;
        let hello_req = Self::get_hello_req(hello_onion).await?;

        let mut guard = context.lock().await;
        let context_locked = &mut *guard;
        let secret = context_locked.crypto.gen_secret();
        
        let pub_key = secret.public_key();
        let channel = Arc::new(OnionChannel::new(secret.symmetric_cipher(hello_req.public_key)));

        let mut circuit_id = None;
        match hello_req.client_type {
            ClientType::Consumer => {
                circuit_id = Some(context_locked.circ_id_generator.get_uid());
                context_locked.circuits.insert(circuit_id.unwrap(), channel.clone());
            },
            ClientType::Relay => {
                context_locked.tunnels.insert(stream.peer_addr()?, channel.clone());
            }
        }
        drop(guard);
        
        let hello_response = Self::generate_hello_response(pub_key, circuit_id);
        Self::send_onion(hello_response, channel.symmetric_cipher(), &stream).await?;
        
        let mut reader = reader.with_cipher(channel.symmetric_cipher());

        let onion = reader.read().await?;

        let mut guard = context.lock().await;
        let context_locked = &mut *guard;

        match onion.target {
            Target::Relay(relay_id) => {
                let relay = context_locked.indexed_relays.iter().find(|relay| relay.id == relay_id).expect("No such relay found");
                let tunnel_stream = TcpStream::connect(relay.addr).await?;

                let crypto = ClientCrypto::new(&relay.pub_key).expect("Failed to generate crypto");
                let secret = crypto.gen_secret();
                let pub_key = secret.public_key();

                let mut writer = RawOnionWriter::new(&tunnel_stream);
                writer.write(
                    Onion {
                            target: Target::Current,
                            circuit_id: None,
                            message: Message::HelloRequest(HelloRequest { client_type: ClientType::Relay, public_key: pub_key })
                }).await?;

                let mut reader = RawOnionReader::new(&tunnel_stream);
                let hello_response = reader.read().await?;

                let symmetric_cipher
                    = if let Message::HelloResponse(peer_key) = hello_response.message {
                        secret.symmetric_cipher(peer_key).expect("Failed to create symmetric cipher")
                    } else {
                        //err?
                        todo!()
                    }
                let channel = OnionChannel::new(symmetric_cipher);

                let mut cursor = Cursor::new(Vec::new());
                RawOnionWriter::new(&mut cursor).with_cipher(channel.symmetric_cipher()).write(onion).await?;

                writer.with_cipher(channel.symmetric_cipher()).write(
                    Onion {
                            target: Target::Current,
                            circuit_id: None,
                            message: Message::Payload(cursor.into_inner())
                }).await?;

                context_locked.tunnels.insert(relay.addr, Arc::new(channel));
            },
            Target::IP(ip) => {
                // I am exit node
                todo!();
            },
            Target::Current => todo!() // err?
        }

        if let Message::Payload(payload) = onion.message {
            todo!();
        } else {
            // err?
            todo!();
        }

        Ok(())
    }

    async fn establish_sender_channel(stream: TcpStream) -> Result<()> {
        let (reader, writer)
            = &mut (RawOnionReader::new(&stream), RawOnionWriter::new(&stream));

        let sender_hello = reader.read().await?;

        Ok(())
    }

    fn establish_receiver_channel() {

    }

    fn generate_hello_response(pub_key: [u8; 96], circ_id: Option<u32>) -> Onion {
        Onion {
            target: Target::Current,
            circuit_id: circ_id,
            message: Message::HelloResponse(pub_key),
        }
    }

    async fn get_hello_req(hello: Onion) -> Result<HelloRequest> {
        if let Message::HelloRequest(req) = hello.message {
            Ok(req)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Expected Hello request"))
        }
    }

    async fn send_onion<T: Write>(onion: Onion, symmetric_cipher: Aes256, writer: T) -> Result<()> {
        RawOnionWriter::new(writer)
            .with_cipher(symmetric_cipher)
            .write(onion)
            .await
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
        }
        
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
