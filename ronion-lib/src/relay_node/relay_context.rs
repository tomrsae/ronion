use std::{collections::HashMap, net::SocketAddr};

use crate::{uid_generator::UIDGenerator, crypto::ServerCrypto};

use super::{channel::Channel};

pub struct RelayContext {
    pub circuits: HashMap<u32, Channel>,
    pub tunnels: HashMap<SocketAddr, Channel>,
    pub circ_id_generator: UIDGenerator,
    pub crypto: ServerCrypto
}

impl RelayContext {
    pub fn new() -> Self {
        Self {
            circuits: HashMap::new(),
            tunnels: HashMap::new(),
            circ_id_generator: UIDGenerator::new(10),
            crypto: ServerCrypto::new()
        }
    }
}
