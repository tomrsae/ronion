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

use super::relay_context::RelayContext;

pub struct RelayNode {
    ip: IpAddr,
    port: u16,
    context: Arc<Mutex<RelayContext>>
}

impl RelayNode {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip: ip,
            port: port,
            context: Arc::new(Mutex::new(RelayContext::new()))
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
            let handler_future
                = async {
                    Self::handle_connection(stream.expect("Failed to read from stream"))
                    .await
                    .expect("Failed to handle connection")
                };
            
            task::spawn(handler_future);
        }
    }

    async fn handle_connection(stream: TcpStream) -> Result<()> {
        Ok(())
    }


}