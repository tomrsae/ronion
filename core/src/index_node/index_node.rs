use async_std::{
    io::{Error, ErrorKind, Result},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    prelude::*,
    sync::{Arc, Mutex},
    task,
};

use crate::protocol::{
    io::{RawOnionReader, RawOnionWriter},
    onion::{Message, Onion, Relay, Target},
};

use super::index_context::IndexContext;

pub struct IndexNode {
    ip: IpAddr,
    port: u16,
    context: Arc<Mutex<IndexContext>>,
}

impl IndexNode {
    // Returns a new IndexNode object
    // param ip: The IP address this index node should bind to
    // param port: The port this index node should listen on
    // param signing_key_pair: The signing key pair used to generate a cryptography context
    pub fn new(ip: IpAddr, port: u16, signing_key_pair: [u8; 64]) -> Self {
        Self {
            ip: ip,
            port: port,
            context: Arc::new(Mutex::new(IndexContext::new(signing_key_pair))),
        }
    }

    // Starts the IndexNode server, causing it to listen to the socket address specified in IndexNode::new()
    pub fn start(&self) {
        let socket = SocketAddr::new(self.ip, self.port);
        let listen_future = self.listen(socket);

        task::block_on(listen_future); // bytte til async?
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
    // param context: Index node context required for management of relays, id generation and cryptography in a static context
    async fn handle_connection(stream: TcpStream, context: Arc<Mutex<IndexContext>>) -> Result<()> {
        let mut reader = RawOnionReader::new(&stream);
        let mut writer = RawOnionWriter::new(&stream);

        let hello = reader.read().await?;
        let peer_key = Self::get_peer_key(hello)?;

        let secret = {
            let mut guard = context.lock().await;
            let context_locked = &mut *guard;

            context_locked.crypto.gen_secret()
        };

        writer
            .write(Onion {
                circuit_id: None,
                message: Message::HelloResponse(secret.public_key()),
                target: Target::Current,
            })
            .await?;

        let symmetric_cipher = secret.symmetric_cipher(peer_key);
        let peer_addr = stream.peer_addr()?;
        let mut reader = reader.with_cipher(symmetric_cipher.clone());
        let mut writer = writer.with_cipher(symmetric_cipher);

        loop {
            let in_onion = match reader.read().await {
                Ok(in_onion) => in_onion,
                Err(err) => match err.kind() {
                    ErrorKind::UnexpectedEof => return Ok(()),
                    _ => return Err(err),
                },
            };
            let out_onion = Self::handle_onion(in_onion, peer_addr, context.clone()).await?;
            writer.write(out_onion).await?;
        }
    }

    fn get_peer_key(hello: Onion) -> Result<[u8; 32]> {
        if let Message::HelloRequest(req) = hello.message {
            Ok(req.public_key)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Expected Hello request"))
        }
    }

    // Reads the contents of an onion and creates an appropriate response onion
    // param onion: The onion to read the contens of
    // param peer_addr: The socket address of the peer who sent the onion
    // param context: Index node context required for management of relays, id generation and cryptography in a static context
    // param peer_key: The public key of the peer who sent the onion
    async fn handle_onion(
        onion: Onion,
        peer_addr: SocketAddr,
        context: Arc<Mutex<IndexContext>>,
    ) -> Result<Onion> {
        let mut guard = context.lock().await;
        let context_locked = &mut *guard;

        let reply = match onion.message {
            Message::GetRelaysRequest() => Onion {
                target: Target::Current,
                circuit_id: Some(context_locked.circ_id_generator.get_uid()),
                message: Message::GetRelaysResponse(context_locked.available_relays.clone()),
            },
            Message::RelayPingRequest(request) => {
                let relay_addr = SocketAddr::new(peer_addr.ip(), request.port);

                let existing_relay = context_locked
                    .available_relays
                    .iter()
                    .find(|relay| relay.addr == relay_addr);

                if existing_relay.is_none() {
                    let id = context_locked.relay_id_generator.get_uid();
                    println!("Registered relay: {} @ {:?}", id, relay_addr);
                    context_locked.available_relays.push(Relay {
                        id,
                        addr: relay_addr,
                        pub_key: request.signing_public,
                    });
                }

                Onion {
                    target: Target::Current,
                    circuit_id: None,
                    message: Message::RelayPingResponse(),
                }
            }
            _ => Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::Close(Some("Invalid request".to_string())),
            },
        };

        Ok(reply)
    }
}
