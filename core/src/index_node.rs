use async_std::{
    task,
    net::{IpAddr, SocketAddr, TcpListener, Incoming, TcpStream}
};

use std::future::Future;
use async_std::prelude::*;

pub struct IndexNode {
    ip: IpAddr,
    //available_relays: Vec<_> // BLOCKED: relay node struct or some other addressable tuple
}

impl IndexNode {
    pub fn new(ip: IpAddr) -> IndexNode {
        IndexNode {
            ip: ip,
            //available_relays: Vec::new()
        }
    }

    async fn listen_for_relay(&self, port: u16) {
        self.listen(port, |stream| async {
            // handle relay registration
            // BLOCKED: ROnion protocol
        }).await
    }
    
    async fn listen_for_consumer(&self, port: u16) {
        self.listen(port, |stream| async {
            // handle consumer request
            // BLOCKED: ROnion protocol
        }).await
    }
    
    async fn listen<F>(&self, port: u16, connection_handler: impl Fn(TcpStream) -> F)
        where F: Future + Send + 'static
    {
        let socket = SocketAddr::new(self.ip, port);
        let listener = TcpListener::bind(socket).await.expect("Failed to bind to socket");
    
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            task::spawn(connection_handler(stream.expect("Failed to receive connection")));
        }
    }
}