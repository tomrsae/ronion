use shadowsocks::relay::socks5::{
    self, Command, HandshakeResponse, TcpRequestHeader, TcpResponseHeader, SOCKS5_AUTH_METHOD_NONE,
};
use shadowsocks::relay::Address;
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
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use core::consumer_node::consumer::Consumer;

pub struct Proxy {
    consumer: Arc<Mutex<Consumer>>,
}

impl Proxy {
    pub async fn new() -> Self {
        let index_addr = "const ip:port";
        let consumer = Arc::new(Mutex::new(Consumer::new(index_addr).await));
        Proxy { consumer }
    }

    pub async fn serve_consumers(&mut self, context: Arc<Context>, svr_cfg: &ServerConfig) -> () {
        let listener = ProxyListener::bind(context, svr_cfg).await.unwrap();
        loop {
            let (stream, target_addr) = listener.accept().await.unwrap();
            println!("-------------NEW CONNECTION");
            let inner = stream.into_inner();
            self.handle_connection(inner, target_addr).await;
        }
    }

    async fn handle_connection(&mut self, mut stream: TcpStream, target_addr: SocketAddr) -> () {
        stream = Proxy::handshake(stream, target_addr).await.unwrap();
        let (mut reader, mut writer) = stream.into_split();
        self.send_consumer(&mut reader); //Seperate sending task
        self.recv_consumer(&mut writer); //Seperate recieving task
    }

    async fn handshake(mut stream: TcpStream, target_addr: SocketAddr) -> io::Result<TcpStream> {
        println!("-------------NEW STREAM");
        let handshake_req = socks5::HandshakeRequest::read_from(&mut stream)
            .await
            .unwrap();

        println!("Req: {:?}", handshake_req.methods);

        for method in handshake_req.methods.iter() {
            match *method {
                socks5::SOCKS5_AUTH_METHOD_NONE => {
                    println!("MONKE");
                    let handshake_resp = HandshakeResponse::new(SOCKS5_AUTH_METHOD_NONE);
                    handshake_resp.write_to(&mut stream).await.unwrap();
                    break;
                }
                _ => {
                    panic!("got unexpected method {}", method);
                }
            }
        }

        let header = match TcpRequestHeader::read_from(&mut stream).await {
            Ok(h) => h,
            Err(err) => {
                println!(
                    "failed to get TcpRequestHeader: {}, peer: {}",
                    err, target_addr
                );
                let rh =
                    TcpResponseHeader::new(err.as_reply(), Address::SocketAddress(target_addr));
                rh.write_to(&mut stream).await.unwrap();
                return Err(err.into());
            }
        };

        println!("Header: {:?}", header);

        match header.command {
            Command::TcpConnect => {
                println!("CONNECT {}", target_addr);

                let header = TcpResponseHeader::new(
                    socks5::Reply::Succeeded,
                    Address::SocketAddress(target_addr),
                );
                header.write_to(&mut stream).await.unwrap();
            }
            _ => {
                panic!("got unexpected command {:?}", header.command);
            }
        }

        return Ok(stream);
    }

    async fn send_consumer(&mut self, stream: &mut OwnedReadHalf) -> () {
        loop {
            let mut payload = [0u8; 1024];

            stream.read(&mut payload).await.unwrap();

            let mut guard = self.consumer.lock().unwrap();
            let consumer_locked = &mut *guard;
            consumer_locked.send_message(payload.to_vec()).await;
        }
    }

    async fn recv_consumer(&mut self, stream: &mut OwnedWriteHalf) -> () {
        loop {
            let mut guard = self.consumer.lock().unwrap();
            let consumer_locked = &mut *guard;
            let payload = consumer_locked.recv_message().await;
            drop(guard);
            stream.write(&payload).await.unwrap();
        }
    }
}
