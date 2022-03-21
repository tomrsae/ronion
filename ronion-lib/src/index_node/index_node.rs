use async_std::{
    task,
    sync::{Arc,Mutex},
    prelude::*,
    net::{IpAddr, SocketAddr, TcpListener, TcpStream}, io::{ReadExt},
    io::Result
};

use crate::{
    crypto::Secret,
    protocol::{
        io::{OnionReader, OnionWriter},
        onion::Onion
    }, relay_node::RelayNode
};

use super::index_context::IndexContext;

pub struct IndexNode {
    ip: IpAddr,
    port: u16,
    context: Arc<Mutex<IndexContext>>
}

impl IndexNode {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        IndexNode {
            ip: ip,
            port: port,
            context: Arc::new(Mutex::new(IndexContext::new()))
        }
    }

    pub fn start(&self) {
        let socket = SocketAddr::new(self.ip, self.port);
        let listen_future = self.listen(socket);
        
        task::block_on(listen_future);
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
        let (reader, writer) = &mut (&stream, &stream);
        
        let mut peer_key_buf = [0u8; 32];
        reader.read_exact(&mut peer_key_buf).await?;
        
        let secret = Secret::new(peer_key_buf);
        let symmetric_cipher = secret.gen_symmetric_cipher();
        let received_onion = OnionReader::new(reader, symmetric_cipher.clone()).read().await?;
        
        let onion = IndexNode::handle_onion(received_onion, context)?;
        
        OnionWriter::new(writer, symmetric_cipher).write(onion).await?;
        
        Ok(())
    }
    
    fn handle_onion(onion: Onion, context: Arc<Mutex<IndexContext>>) -> Result<Onion> {
        panic!("not yet implemented");
    }

    // async fn handle_relay(onion: Onion, context: Arc<Mutex<IndexContext>>) -> Result<Onion> {
    //     let guard = context.lock().await;
    //     let context_locked = &mut *guard;
        
    //     context_locked.available_relays.push(RelayNode {

    //     });

    //     Onion {
    //         target: ,
    //         payload: 
    //     }
    // }
    
    // async fn handle_consumer(onion: Onion) -> Onion {
    //     // BLOCKED: ROnion protocol
    // }
}