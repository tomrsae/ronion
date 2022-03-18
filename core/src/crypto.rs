use aes::{
    cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit},
    Aes256,
};
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

pub struct Secret {
    secret: EphemeralSecret,
    peer_public_key: [u8; 32],
}

pub trait SymmetricCipher {
    fn encrypt(&self, block: &mut [u8]);
    fn decrypt(&self, block: &mut [u8]);
}

impl SymmetricCipher for Aes256 {
    fn encrypt(&self, block: &mut [u8]) {
        let mut array = GenericArray::from_mut_slice(block);
        self.encrypt_block(&mut array);
    }
    fn decrypt(&self, block: &mut [u8]) {
        let mut array = GenericArray::from_mut_slice(block);
        self.decrypt_block(&mut array);
    }
}

impl Secret {
    pub fn new(peer_public_key: [u8; 32]) -> Self {
        Secret {
            secret: EphemeralSecret::new(OsRng),
            peer_public_key,
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
                .diffie_hellman(&PublicKey::from(self.peer_public_key))
                .as_bytes(),
        )
    }

    fn gen_cipher(byte_key: &[u8; 32]) -> Aes256 {
        Aes256::new_from_slice(byte_key).expect("Invalid key length")
    }
}
