use aes::cipher::KeyInit;
use aes::Aes256;
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

pub struct Secret {
    secret: EphemeralSecret,
}

impl Secret {
    pub fn new() -> Secret {
        Secret {
            secret: EphemeralSecret::new(OsRng),
        }
    }

    pub fn create_secrets(n: usize) -> Vec<Secret> {
        let mut secrets = Vec::with_capacity(n);
        for _ in 0..n {
            secrets.push(Secret::new())
        }
        secrets
    }

    pub fn gen_pub_key(&mut self) -> PublicKey {
        PublicKey::from(&self.secret)
    }

    pub fn gen_shared_key(self, pub_key: &PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(pub_key)
    }

    pub fn gen_cipher(&mut self) -> Aes256 {
        let key = self.gen_pub_key();
        match Aes256::new_from_slice(key.as_bytes()) {
            Ok(v) => v,
            Err(e) => panic!("Invalid key length: {}", e),
        }
    }
}
