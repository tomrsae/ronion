use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use shadowsocks::context::Context;
use shadowsocks::relay::tcprelay::proxy_listener::ProxyListener;
use shadowsocks::relay::tcprelay::ProxyServerStream;
use shadowsocks::{self, ServerConfig};

use crate::{
    consumer_node::consumer::Consumer,
    protocol::{
        io::{OnionReader, OnionWriter},
        onion::{Onion, Target},
    },
};
pub struct Proxy {
    consumer: Consumer,
}

impl Proxy {
    async fn new(n: usize) -> Self {
        let consumer = Consumer::new(n).await;
        Proxy { consumer }
    }

    pub async fn listen_consumers(&self, context: Arc<Context>, svr_cfg: &ServerConfig) -> () {
        let listener = ProxyListener::bind(context, svr_cfg).await.unwrap();
        loop {
            let (mut stream, socket) = listener.accept().await.unwrap();
            self.handle_connection(stream.get_mut());
        }
    }

    async fn handle_connection(&self, stream: &mut TcpStream) -> () {
        panic!("consumer client not yet implemented")
    }

    async fn send_consumer(&mut self, stream: &mut TcpStream) -> () {
        let mut payload = [0u8; 1024];

        stream.read(&mut payload).await.unwrap();

        self.consumer.send_message(payload.to_vec()).await;
    }

    async fn recv_consumer(&mut self, stream: &mut TcpStream) -> () {
        let payload = self.consumer.recv_message().await;

        stream.write(&payload).await.unwrap();
    }

    async fn relay() -> () {}
}
