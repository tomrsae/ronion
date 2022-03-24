pub mod proxy;

use std::{env, net::SocketAddr, str::FromStr};

use proxy::Proxy;
use shadowsocks::{config::ServerType, context::Context, crypto::v1::CipherKind, ServerConfig};

#[tokio::main]
async fn main() {
    //Argument order: ip:port (incoming), password, encryption-method, "start"
    let args: Vec<String> = env::args().collect();
    let (port, pw, cipmet) = parse_input(args);

    let mut proxy: Proxy;
    let context = Context::new_shared(ServerType::Local);
    let svr_cfg = ServerConfig::new(port, pw, cipmet);

    proxy = Proxy::new().await;
    proxy.serve_consumers(context, &svr_cfg).await;
}

fn parse_input(args: Vec<String>) -> (SocketAddr, String, CipherKind) {
    let addr = args[1].parse::<SocketAddr>().unwrap();
    let pw = args[2].clone();
    let cipmet = CipherKind::from_str(&args[3]).unwrap();

    (addr, pw, cipmet)
}
