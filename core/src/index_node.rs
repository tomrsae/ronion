use async_std::{
    task,
    prelude::*,
    net::{IpAddr, SocketAddr, TcpListener, TcpStream}
};

use std::future::Future;

use crate::relay_node::RelayNode;

pub struct IndexNode {
    ip: IpAddr,
    available_relays: Vec<RelayNode>
}

impl IndexNode {
    pub fn new(ip: IpAddr) -> Self {
        IndexNode {
            ip: ip,
            available_relays: Vec::new()
        }
    }

    async fn handle_relay(available_relays: &mut Vec<RelayNode>) {
            // BLOCKED: ROnion protocol

            //available_relays.push(RelayNode::new())
    }

    async fn handle_consumer() {
        // BLOCKED: ROnion protocol
    }
    
    async fn listen(&self, port: u16)
    {
        let socket = SocketAddr::new(self.ip, port);
        let listener = TcpListener::bind(socket).await.expect("Failed to bind to socket");
    
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            // check if request is coming from relay or consumer
            // and run the appropriate handler
            let stream = stream.expect("Failed to receive connection");
            //task::spawn(connection_handler(stream));
        }
    }
}