use aes::{
    cipher::{BlockEncrypt, KeyInit, BlockDecrypt, generic_array::GenericArray}, 
    Aes256
};
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

pub struct Secret {
    secret: EphemeralSecret,
    incoming_key: [u8; 32],
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

    pub fn gen_shared_key(self) -> SharedSecret {
        self.secret
            .diffie_hellman(&PublicKey::from(self.incoming_key))
    }

    //For circuit creation
    pub fn gen_circuit_cipher(&self) -> Aes256 {
        Secret::gen_cipher(&self.incoming_key)
    }

    //For messaging
    pub fn gen_secret_cipher(self) -> Aes256 {
        Secret::gen_cipher(self.gen_shared_key().as_bytes())
    }

    fn gen_cipher(byte_key: &[u8; 32]) -> Aes256 {
        Aes256::new_from_slice(byte_key).expect("Invalid key length")
    }
}
