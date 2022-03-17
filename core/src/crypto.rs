use aes::{cipher::KeyInit, Aes256};
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
        (0..n).into_iter().map(|_| Secret::new()).collect()
    }

    pub fn gen_pub_key(&mut self) -> PublicKey {
        PublicKey::from(&self.secret)
    }

    pub fn gen_shared_key(self, pub_key: &PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(pub_key)
    }

    pub fn gen_cipher(&mut self) -> Aes256 {
        let key = self.gen_pub_key();
        Aes256::new_from_slice(key.as_bytes()).expect("Invalid key length")
    }
}
