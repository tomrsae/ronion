use shadowsocks::{
    self,
    context::Context,
    relay::tcprelay::{
        proxy_listener::ProxyListener,
        proxy_stream::{ProxyServerStreamReadHalf, ProxyServerStreamWriteHalf},
        ProxyServerStream,
    },
    ServerConfig,
};

use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::consumer_node::consumer::Consumer;

pub struct Proxy {
    consumer: Consumer,
}

impl Proxy {
    pub async fn new(n: usize) -> Self {
        let index_pub_key: [u8; 32] = [
            //Hardcoded public index key
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let consumer = Consumer::new(n, index_pub_key).await;
        Proxy { consumer }
    }

    pub async fn listen_consumers(&mut self, context: Arc<Context>, svr_cfg: &ServerConfig) -> () {
        let listener = ProxyListener::bind(context, svr_cfg).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let _ = self.handle_connection(stream);
        }
    }

    async fn handle_connection(&mut self, stream: ProxyServerStream<TcpStream>) -> () {
        let (mut reader, mut writer) = stream.into_split();
        let _ = self.send_consumer(&mut reader);
        let _ = self.recv_consumer(&mut writer);
    }

    async fn send_consumer(&mut self, stream: &mut ProxyServerStreamReadHalf<TcpStream>) -> () {
        loop {
            let mut payload = [0u8; 1024];

            stream.read(&mut payload).await.unwrap();

            self.consumer.send_message(payload.to_vec()).await
        }
    }

    async fn recv_consumer(&mut self, stream: &mut ProxyServerStreamWriteHalf<TcpStream>) -> () {
        loop {
            let payload = self.consumer.recv_message().await;

            stream.write(&payload).await.unwrap();
        }
    }
}
