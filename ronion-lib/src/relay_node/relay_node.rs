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

use super::{circuit::Circuit, circuit_connection::CircuitConnection, relay_context::RelayContext};

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
        let incoming = CircuitConnection {
            stream: incoming_stream,
        };

        if let Ok(outgoing_stream) = TcpStream::connect("localhost:7070").await {
            // fix
            let circuit = Arc::new(Circuit {
                id: 2_u32, //idk
                outgoing: CircuitConnection {
                    stream: outgoing_stream,
                },
                incoming: incoming,
            });

            //task::spawn(circuit.activate());

            let mut guard = context.lock().await;
            let context_locked = &mut *guard;

            context_locked.circuits.push(circuit);
        } else {
            // err?
        }

        Ok(())
    }
}
