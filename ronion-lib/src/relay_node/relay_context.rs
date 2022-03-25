use std::{collections::HashMap, net::SocketAddr};

use crate::{uid_generator::UIDGenerator, crypto::ServerCrypto};

use super::{circuit::Circuit, tunnel::Tunnel};

pub struct RelayContext {
    pub circuits: HashMap<u32, Circuit>,
    pub tunnels: HashMap<SocketAddr, Tunnel>,
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
