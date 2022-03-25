use async_std::{
    task,
    sync::{Arc, Mutex},
    prelude::*,
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    io::{Result, Error, ErrorKind}
};

use crate::{
    protocol::{
        io::{RawOnionWriter, RawOnionReader},
        onion::{Onion, Target, Message, Relay}
    }
};

use super::index_context::IndexContext;

pub struct IndexNode {
    ip: IpAddr,
    port: u16,
    context: Arc<Mutex<IndexContext>>
}

impl IndexNode {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip: ip,
            port: port,
            context: Arc::new(Mutex::new(IndexContext::new()))
        }
    }

    pub fn start(&self) {
        let socket = SocketAddr::new(self.ip, self.port);
        let listen_future = self.listen(socket);
        
        task::block_on(listen_future); // bytte til async?
    }

    
    async fn listen(&self, socket: SocketAddr) {
        let listener = TcpListener::bind(socket).await.expect("Failed to bind to socket");
        
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let context = self.context.clone();
            let handler_future
                = async {
                    Self::handle_connection(stream.expect("Failed to read from stream"), context)
                    .await
                    .expect("Failed to handle connection")
                };
            
            task::spawn(handler_future);
        }
    }
    
    async fn handle_connection(stream: TcpStream, context: Arc<Mutex<IndexContext>>) -> Result<()> {
        let mut reader = RawOnionReader::new(&stream);

        let hello = reader.read().await?;
        let peer_key = Self::get_peer_key(hello).await?;
        
        let secret;
        {
            let mut guard = context.lock().await;
            let context_locked = &mut *guard;

            secret = context_locked.crypto.gen_secret();
        }

        let symmetric_cipher = secret.symmetric_cipher(peer_key);
        let received_onion = reader.with_cipher(symmetric_cipher.clone()).read().await?;

        let response = Self::handle_onion(received_onion, stream.peer_addr()?, context, peer_key).await?;
        
        RawOnionWriter::new(&stream).with_cipher(symmetric_cipher).write(response).await
    }

    async fn get_peer_key(hello: Onion) -> Result<[u8; 32]> {
        if let Message::HelloRequest(req) = hello.message {
            Ok(req.public_key)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Expected Hello request"))
        }
    }
    
    async fn handle_onion(onion: Onion, peer_addr: SocketAddr, context: Arc<Mutex<IndexContext>>, peer_key: [u8; 32]) -> Result<Onion> {
        let mut guard = context.lock().await;
        let context_locked = &mut *guard;

        let reply = match onion.message {
            Message::GetRelaysRequest() => {
                Onion {
                    target: Target::Current,
                    circuit_id: Some(context_locked.circ_id_generator.get_uid()),
                    message: Message::GetRelaysResponse(context_locked.available_relays.clone())
                }
            },
            Message::RelayPingRequest() => {
                let existing_relay = context_locked.available_relays.iter().find(|relay| relay.addr == peer_addr);
                
                if existing_relay.is_none() {
                    context_locked.available_relays.push(Relay {
                        id: context_locked.relay_id_generator.get_uid(),
                        addr: peer_addr,
                        pub_key: peer_key
                    });
                }
                
                Onion {
                    target: Target::Current,
                    circuit_id: None,
                    message: Message::RelayPingResponse()
                }
            },
            _ => Onion {
                    target: Target::Current,
                    circuit_id: None,
                    message: Message::Close(Some("Invalid request".to_string()))
                }
        };

        Ok(reply)
    }
}
