use std::{collections::HashMap};

use crate::{uid_generator::UIDGenerator, crypto::ServerCrypto};

use super::circuit::Circuit;

pub struct RelayContext {
    pub circuits: HashMap<u32, Circuit>,
    pub circ_id_generator: UIDGenerator,
    pub crypto: ServerCrypto
}

impl RelayContext {
    pub fn new() -> Self {
        Self {
            circuits: HashMap::new(),
            circ_id_generator: UIDGenerator::new(10),
            crypto: ServerCrypto::new()
        }
    }
}
