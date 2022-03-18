use aes::{cipher::KeyInit, Aes256};
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

pub struct Secret {
    secret: EphemeralSecret,
    incoming_key: [u8; 32],
}

impl Secret {
    pub fn new(incoming_key: [u8; 32]) -> Self {
        Secret {
            secret: EphemeralSecret::new(OsRng),
            incoming_key,
        }
    }

    pub fn create_secrets(n: usize, recv_keys: Vec<[u8; 32]>) -> Vec<Secret> {
        (0..n)
            .into_iter()
            .map(|i| Secret::new(recv_keys[i]))
            .collect()
    }

    //Should be sent to the other peer. Must be generated/used before shared key
    //since shared key consumes the secret.
    pub fn gen_pub_key(&self) -> PublicKey {
        PublicKey::from(&self.secret)
    }

    //For messaging
    pub fn gen_symmetric_cipher(self) -> Aes256 {
        Secret::gen_cipher(
            self.secret
                .diffie_hellman(&PublicKey::from(self.incoming_key))
                .as_bytes(),
        )
    }

    fn gen_cipher(byte_key: &[u8; 32]) -> Aes256 {
        Aes256::new_from_slice(byte_key).expect("Invalid key length")
    }
}
