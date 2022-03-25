use std::rc::Rc;

use aes::Aes256;
use async_std::{
    io::{Error, ErrorKind, Result, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    prelude::*,
    sync::{Arc, Mutex},
    task,
};

use crate::{
    protocol::{
        io::{RawOnionReader, RawOnionWriter},
        onion::{Message, Onion, Target, HelloRequest, ClientType},
    },
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
        incoming_stream: TcpStream,
        context: Arc<Mutex<RelayContext>>,
    ) -> Result<()> {
        let mut reader = RawOnionReader::new(&incoming_stream);

        let hello_onion = reader.read().await?;
        let hello_req = Self::get_hello_req(hello_onion).await?;

        let mut guard = context.lock().await;
        let context_locked = &mut *guard;
        let secret = context_locked.crypto.gen_secret();
        
        let pub_key = secret.public_key();
        let channel = Arc::new(OnionChannel::new(incoming_stream, secret.symmetric_cipher(hello_req.public_key)));

        let mut circuit_id = None;
        match hello_req.client_type {
            ClientType::Consumer => {
                circuit_id = Some(context_locked.circ_id_generator.get_uid());
                context_locked.circuits.insert(circuit_id.unwrap(), channel.clone());
            },
            ClientType::Relay => {
                context_locked.tunnels.insert(channel.stream.peer_addr()?, channel.clone());
            }
        }
        drop(guard);
        
        let hello_response = Self::generate_hello_response(pub_key, circuit_id);
        Self::send_onion(hello_response, channel.symmetric_cipher(), &channel.stream).await?;

        channel.open().await;

        Ok(())
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
}
