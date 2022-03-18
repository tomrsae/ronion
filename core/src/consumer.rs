use std::net::TcpStream;

use crate::crypto::Secret;
use aes::{cipher::KeyInit, Aes256};
use x25519_dalek::{PublicKey, SharedSecret};
pub struct Consumer {
    public_keys: Vec<[u8; 32]>,
    circuit_ciphers: Vec<Aes256>,
    secret_ciphers: Vec<Aes256>,
}

impl Consumer {
    fn new(n: usize, recv_keys: Vec<[u8; 32]>) -> Self {
        let mut secrets = Secret::create_secrets(n, recv_keys);
        let mut public_keys = Vec::<[u8; 32]>::with_capacity(n);
        let mut secret_ciphers = Vec::<Aes256>::with_capacity(n);
        let mut circuit_ciphers = Vec::<Aes256>::with_capacity(n);
        // let mut secret_keys = Vec::<SharedSecret>::with_capacity(n);

        for i in 0..(n - 1) {
            let secret = secrets.remove(i);
            public_keys.push(secret.gen_pub_key().as_bytes().to_owned());
            circuit_ciphers.push(secret.gen_circuit_cipher());
            secret_ciphers.push(secret.gen_secret_cipher());
            //secret_keys.push(secret.gen_shared_key(&PublicKey::from(recv_keys[i])));
        }

        Consumer {
            public_keys,
            circuit_ciphers,
            secret_ciphers,
            // secret_keys,
        }
    }

    pub fn dial_index() -> TcpStream {
        let addr = ""; //Decide addresses to use/how to find address?
        TcpStream::connect(addr).expect("")
    }

    pub fn encrypt_circuit(&self, index: ) -> () {
        self.circuit_ciphers.
    }
}
