use aes::Aes256;
use async_std::{io::{Result, Read}, net::TcpStream};

use crate::{crypto::ServerSecret, protocol::onion::Onion};

use super::circuit_connection::CircuitConnection;

#[derive(Clone)]
pub struct Circuit {
    pub id: u32,
    pub peer_key: [u8; 32],
    pub symmetric_cipher: Aes256,
    pub public_key: [u8; 96]
}

impl Circuit {
    pub fn new(id: u32, secret: ServerSecret, peer_key: [u8; 32]) -> Self {
        Self {
            id: id,
            peer_key: peer_key,
            public_key: secret.public_key(),
            symmetric_cipher: secret.symmetric_cipher(peer_key)
        }
    }

    pub fn symmetric_cipher(&self) -> Aes256 {
        self.symmetric_cipher.clone()
    }
}