use std::{collections::HashMap, net::SocketAddr, rc::Rc, sync::Arc};

use crate::{uid_generator::UIDGenerator, crypto::ServerCrypto, protocol::onion::Relay};

use super::{channel::OnionChannel};

pub struct RelayContext {
    pub circuits: HashMap<u32, Arc<OnionChannel>>,
    pub tunnels: HashMap<SocketAddr, Arc<OnionChannel>>,
    pub indexed_relays: Vec<Relay>,
    pub circ_id_generator: UIDGenerator,
    pub crypto: ServerCrypto
}

impl RelayContext {
    pub fn new() -> Self {
        Self {
            circuits: HashMap::new(),
            tunnels: HashMap::new(),
            indexed_relays: Vec::new(),
            circ_id_generator: UIDGenerator::new(10),
            crypto: ServerCrypto::new()
        }
    }
}
