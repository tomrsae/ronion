use async_std::{
    task,
    prelude::*,
    net::{IpAddr, SocketAddr, TcpListener, TcpStream}
};

use std::future::Future;

use crate::{relay_node::RelayNode, protocol::{Onion, OnionReader, OnionWriter}, crypto::Secret};

pub struct IndexNode {
    ip: IpAddr,
    port: u16,
    available_relays: Vec<RelayNode>
}

impl IndexNode {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        IndexNode {
            ip: ip,
            port: port,
            available_relays: Vec::new()
        }
    }

    pub fn start(&self) {
        let socket = SocketAddr::new(self.ip, self.port);
        let listen_future = self.listen(socket);
        
        task::block_on(listen_future);
    }

    // async fn handle_relay(onion: Onion, available_relays: &mut Vec<RelayNode>) -> Onion {
    //         // BLOCKED: ROnion protocol

    //         //available_relays.push(RelayNode::new())
    // }

    // async fn handle_consumer(onion: Onion) -> Onion {
    //     // BLOCKED: ROnion protocol
    // }
    
    async fn listen(&self, socket: SocketAddr) {
        let listener = TcpListener::bind(socket).await.expect("Failed to bind to socket");
    
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            // check if request is coming from relay or consumer
            // and run the appropriate handler
            let stream = stream.expect("Failed to receive connection");
            let (reader, writer) = &mut (&stream, &stream);

            let mut pub_key_buf = [0u8; 32];
            reader.read(&mut pub_key_buf).await.expect("Failed to read public key");

            let secret = Secret::new(pub_key_buf);
            let received_onion = OnionReader::new(reader, secret.gen_symmetric_cipher());

            //task::spawn(connection_handler(stream));
        }
    }
}