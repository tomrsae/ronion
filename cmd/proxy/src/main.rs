pub mod proxy;

use proxy::Proxy;
use shadowsocks::{config::ServerType, context::Context, crypto::v1::CipherKind, ServerConfig};
use std::{env, net::SocketAddr, path::Path, str::FromStr};

#[tokio::main]
async fn main() {
    //Argument order: ip:port (incoming), password, encryption-method, "start"
    let args: Vec<String> = env::args().collect();
    let (host_addr, pw, index_addr) = parse_input(args);

    let mut proxy: Proxy;
    let context = Context::new_shared(ServerType::Local);
    let svr_cfg = ServerConfig::new(host_addr, pw, CipherKind::AES_256_GCM);
    println!("proxinu");
    proxy = Proxy::new(index_addr).await;
    println!("work?");
    proxy.serve_consumers(context, &svr_cfg).await;
}

fn parse_input(args: Vec<String>) -> (SocketAddr, String, String) {
    let host_addr = args[1].parse::<SocketAddr>().unwrap();
    let pw = args[2].clone();
    let index_addr = args[3].clone();

    (host_addr, pw, index_addr)
}
