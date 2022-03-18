use async_std::{
    task,
    prelude::*,
    net::{IpAddr, SocketAddr, TcpListener, TcpStream}, io::{ReadExt},
    io::Result
};

use crate::{
    relay_node::RelayNode,
    crypto::Secret,
    protocol::{
        io::{OnionReader, OnionWriter},
        onion::Onion
    }
};

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
        let (reader, writer) = &mut (&stream, &stream);

        let mut peer_key_buf = [0u8; 32];
        reader.read_exact(&mut peer_key_buf).await?;

        let secret = Secret::new(peer_key_buf);
        let symmetric_cipher = secret.gen_symmetric_cipher();
        let received_onion = OnionReader::new(reader, symmetric_cipher.clone()).read().await?;

        let onion = IndexNode::handle_onion(received_onion)?;
        
        OnionWriter::new(writer, symmetric_cipher).write(onion).await?;

        Ok(())
    }

    fn handle_onion(onion: Onion) -> Result<Onion> {
        panic!("not yet implemented");
    }
}