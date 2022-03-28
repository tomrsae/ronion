use std::{collections::HashMap, net::SocketAddr, rc::Rc, sync::Arc};

use crate::crypto::Aes256;
use async_std::net::TcpStream;

use crate::{crypto::ServerCrypto, protocol::onion::Relay, uid_generator::UIDGenerator};

use super::tunnel::OnionTunnel;

pub struct RelayContext {
    pub circuits: HashMap<u32, Arc<Circuit>>, //HashMap<u32, Arc<OnionChannel>>,
    pub relay_tunnels: HashMap<SocketAddr, Arc<OnionTunnel>>,
    pub indexed_relays: Vec<Relay>,
    pub circ_id_generator: UIDGenerator,
    pub crypto: ServerCrypto,
}

impl RelayContext {
    pub fn new() -> Self {
        Self {
            circuits: HashMap::new(),
            relay_tunnels: HashMap::new(),
            indexed_relays: Vec::new(),
            circ_id_generator: UIDGenerator::new(10),
            crypto: ServerCrypto::new(),
        }
    }
}

pub struct Circuit {
    pub id: u32,
    pub symmetric_cipher: Aes256,
    pub peel_tunnel_addr: SocketAddr,
    pub layer_tunnel_addr: SocketAddr,
    pub endpoint_connection: Option<TcpStream>,
}
