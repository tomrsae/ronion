use aes::Aes256;
use async_std::{
    io::{Error, ErrorKind, Result, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    prelude::*,
    sync::{Arc, Mutex},
    task,
};

use crate::{
    crypto::ServerSecret,
    protocol::{
        io::{RawOnionReader, RawOnionWriter},
        onion::{Message, Onion, Target},
    },
};

use super::{
    circuit::{self, Circuit},
    relay_context::RelayContext,
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

        let hello = reader.read().await?;

        let peer_key = Self::get_peer_key(hello).await?;

        let mut guard = context.lock().await;
        let context_locked = &mut *guard;
        let secret = context_locked.crypto.gen_secret();
        let pub_key = secret.public_key();

        let mut symmetric_cipher = secret.symmetric_cipher(peer_key);
        let mut circuit_id = None;
        if true {
            // replace literal
            // Onion is from consumer, create circuit
            circuit_id = Some(context_locked.circ_id_generator.get_uid());

            let circuit = Circuit {
                id: circuit_id.unwrap(),
                peer_key: peer_key,
                symmetric_cipher: symmetric_cipher,
                public_key: pub_key,
            };

            symmetric_cipher = circuit.symmetric_cipher();
            context_locked.circuits.insert(circuit_id.unwrap(), circuit);
        }
        drop(guard);

        let hello_response = Self::generate_hello_response(pub_key, circuit_id);
        Self::send_onion(hello_response, symmetric_cipher, &incoming_stream).await?;

        Ok(())
    }

    fn generate_hello_response(pub_key: [u8; 96], circ_id: Option<u32>) -> Onion {
        Onion {
            target: Target::Current,
            circuit_id: circ_id,
            message: Message::HelloResponse(pub_key),
        }
    }

    async fn get_peer_key(hello: Onion) -> Result<[u8; 32]> {
        if let Message::HelloRequest(req) = hello.message {
            Ok(req.public_key)
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
