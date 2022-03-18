use aes::{cipher::KeyInit, Aes256};
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

pub struct Secret {
    secret: EphemeralSecret,
    recv_key: [u8; 32],
}

impl Secret {
    pub fn new(recv_key: [u8; 32]) -> Secret {
        Secret {
            secret: EphemeralSecret::new(OsRng),
            recv_key,
        }
    }

    pub fn create_secrets(n: usize, recv_keys: Vec<[u8; 32]>) -> Vec<Secret> {
        (0..n)
            .into_iter()
            .map(|i| Secret::new(recv_keys[i]))
            .collect()
    }

    pub fn gen_pub_key(&mut self) -> PublicKey {
        PublicKey::from(&self.secret)
    }

    pub fn gen_shared_key(self, pub_key: &PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(pub_key)
    }

    pub fn gen_pub_cipher(&mut self) -> Aes256 {
        let key = self.gen_pub_key();
        gen_cipher(key.as_bytes())
    }
    pub fn gen_secret_cipher(&mut self) -> Aes256 {
        gen_cipher(&self.recv_key)
    }
}

fn gen_cipher(byte_key: &[u8; 32]) -> Aes256 {
    Aes256::new_from_slice(byte_key).expect("Invalid key length")
}
