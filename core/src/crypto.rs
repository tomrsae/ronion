use aes::cipher::{generic_array::GenericArray, BlockCipher, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::{Aes256, Aes256Dec, Aes256Enc, Block8};
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

struct Secret {
    secret: EphemeralSecret,
}

impl Secret {
    fn new() -> Secret {
        Secret {
            secret: EphemeralSecret::new(OsRng),
        }
    }

    fn create_secrets(n: usize) -> Vec<Secret> {
        let mut secrets = Vec::with_capacity(n);
        for _ in 0..n {
            secrets.push(Secret::new())
        }
        secrets
    }

    fn gen_pub_key(&mut self) -> PublicKey {
        PublicKey::from(&self.secret)
    }

    fn gen_shared_key(mut self, pub_key: &PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(pub_key)
    }

    fn gen_cipher(&mut self) -> Aes256 {
        let key = self.gen_pub_key();
        match Aes256::new_from_slice(key.as_bytes()) {
            Ok(v) => v,
            Err(e) => panic!("Invalid key length: {}", e),
        }
    }
}
