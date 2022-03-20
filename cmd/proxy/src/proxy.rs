use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;

use shadowsocks::context::Context;
use shadowsocks::relay::tcprelay::proxy_listener::ProxyListener;
use shadowsocks::relay::tcprelay::ProxyServerStream;
use shadowsocks::{self, ServerConfig};

pub struct Proxy;

impl Proxy {
    pub async fn listen<T: Peer<TcpStream>>(context: Arc<Context>, svr_cfg: &ServerConfig) -> () {
        let listener = ProxyListener::bind(context, svr_cfg).await.unwrap();
        loop {
            let (stream, socket) = listener.accept().await.unwrap();
            let test = T::handle_connection(stream, socket);
        }
    }
}

struct StreamType<'a> {
    s: &'a mut TcpStream,
}

#[async_trait(?Send)]
pub trait Peer<S> {
    async fn handle_connection(stream: ProxyServerStream<S>, socket: SocketAddr)
    where
        S: 'async_trait;
}

struct ConsumerClient<S> {
    stream: ProxyServerStream<S>,
    socket: SocketAddr,
}

#[async_trait(?Send)]
impl<'a> Peer<StreamType<'a>> for ConsumerClient<TcpStream> {
    async fn handle_connection(
        stream: ProxyServerStream<StreamType<'a>>,
        socket: SocketAddr,
    ) -> () {
        panic!("consumer client not yet implemented")
    }
}

struct RelayClient<S> {
    stream: ProxyServerStream<S>,
    socket: SocketAddr,
}

#[async_trait(?Send)]
impl<'a> Peer<StreamType<'a>> for RelayClient<TcpStream> {
    async fn handle_connection(
        stream: ProxyServerStream<StreamType<'a>>,
        socket: SocketAddr,
    ) -> () {
        panic!("relay client not yet implemented")
    }
}
