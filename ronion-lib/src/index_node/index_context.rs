use crate::{protocol::onion::Relay, crypto::{ServerCrypto}};

use super::uid_generator::UIDGenerator;

pub struct IndexContext {
    pub available_relays: Vec<Relay>,
    pub circ_id_generator: UIDGenerator,
    pub relay_id_generator: UIDGenerator,
    pub crypto: ServerCrypto
}

impl IndexContext {
    pub fn new() -> Self {
        IndexContext {
            available_relays: Vec::new(),
            circ_id_generator: UIDGenerator::new(10),
            relay_id_generator: UIDGenerator::new(10),
            crypto: ServerCrypto::new()
        }
    }
}