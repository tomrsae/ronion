pub mod proxy;

use std::sync::Arc;

use proxy::Proxy;
use shadowsocks::{context::Context, ServerConfig};

#[tokio::main]
async fn main() {
    let mut proxy: Proxy;
    let context: Arc<Context>;
    let svr_cfg: &ServerConfig;

    proxy = Proxy::new(4).await;
    proxy.serve_consumers(context, svr_cfg).await;
}
