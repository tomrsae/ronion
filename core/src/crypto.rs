use std::sync::Arc;

use aes_gcm::{
    aead::{Aead, NewAead},
    Aes256Gcm, Key, Nonce,
};
use ed25519_dalek::{Keypair, Signature, Signer, Verifier};
use rand_core::{OsRng, RngCore};
use x25519_dalek::{EphemeralSecret, PublicKey};

pub trait SymmetricCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8>;
    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8>;
}

pub struct Aes256 {
    aes: Arc<Aes256Gcm>,
}
impl Aes256 {
    fn new(key: [u8; 32]) -> Aes256 {
        let aes = Arc::new(Aes256Gcm::new(Key::from_slice(&key)));
        Self { aes }
    }
}
impl Clone for Aes256 {
    fn clone(&self) -> Self {
        let aes = self.aes.clone();
        Self { aes }
    }
}
impl SymmetricCipher for Aes256 {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut nonce = [0u8; 12];
        OsRng {}.fill_bytes(&mut nonce);

        let mut ciphertext = self
            .aes
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .expect("encryption failed");

        ciphertext.extend(nonce.iter());

        ciphertext
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let full_len = ciphertext.len();
        let nonce = &ciphertext[full_len - 12..full_len];
        let ciphertext = &ciphertext[..full_len - 12];

        let plaintext = self
            .aes
            .decrypt(Nonce::from_slice(&nonce), ciphertext)
            .expect("decryption failed");

        plaintext
    }
}

#[derive(Debug)]
pub enum KeypairError {
    InvalidData,
}

#[derive(Debug)]
pub enum SignatureError {
    InvalidData,
    InvalidSignature,
}

#[derive(Debug)]
pub enum SigningPublicKeyError {
    InvalidData,
}

pub struct ServerSecret {
    keypair: Keypair,
    secret: EphemeralSecret,
}
impl ServerSecret {
    /// Gets the secret's signed public key.
    pub fn public_key(&self) -> [u8; 96] {
        let key = PublicKey::from(&self.secret).to_bytes();
        let signature = self.keypair.sign(&key).to_bytes();
        let mut target = [0u8; 96];
        for (dst, src) in target.iter_mut().zip(key.iter().chain(signature.iter())) {
            *dst = *src;
        }
        target
    }

    /// Combines secret and peer public key into a SymmetricCipher.
    pub fn symmetric_cipher(self, peer_public: [u8; 32]) -> Aes256 {
        let peer_public = PublicKey::from(peer_public);
        let shared_secret = self.secret.diffie_hellman(&PublicKey::from(peer_public));
        Aes256::new(shared_secret.to_bytes())
    }
}

/// ServerCrypto provides server-side cryptography.
pub struct ServerCrypto {
    keypair: Keypair,
}
impl ServerCrypto {
    /// Creates a ServerCrypto with a random signing keypair.
    pub fn new() -> Self {
        Self {
            keypair: Keypair::generate(&mut OsRng {}),
        }
    }

    /// Creates a ServerCrypto from signing keypair bytes.
    pub fn from_bytes(keypair_bytes: &[u8; 64]) -> Result<Self, KeypairError> {
        let keypair = Keypair::from_bytes(keypair_bytes).map_err(|_| KeypairError::InvalidData)?;
        Ok(Self { keypair })
    }

    /// Converts the ServerCrypto's signing keypair to bytes.
    pub fn to_bytes(&self) -> [u8; 64] {
        self.keypair.to_bytes()
    }

    /// Gets the signing public key.
    pub fn signing_public(&self) -> [u8; 32] {
        self.keypair.public.to_bytes()
    }

    /// Generate a new secret.
    pub fn gen_secret(&self) -> ServerSecret {
        ServerSecret {
            secret: EphemeralSecret::new(&mut OsRng {}),
            keypair: Keypair::from_bytes(&self.keypair.to_bytes()).unwrap(),
        }
    }
}

pub struct ClientSecret {
    verifier: ed25519_dalek::PublicKey,
    secret: EphemeralSecret,
}

impl ClientSecret {
    /// Gets the secrets public key.
    pub fn public_key(&self) -> [u8; 32] {
        PublicKey::from(&self.secret).to_bytes()
    }

    ///Combines secret and public key of peer to a SymmetricCipher.
    pub fn symmetric_cipher(self, peer_public: [u8; 96]) -> Result<Aes256, SignatureError> {
        let key: [u8; 32] = peer_public[0..32].try_into().unwrap();
        let signature =
            Signature::from_bytes(&peer_public[32..96]).map_err(|_| SignatureError::InvalidData)?;
        self.verifier
            .verify(&key, &signature)
            .map_err(|_| SignatureError::InvalidSignature)?;

        let shared_secret = self.secret.diffie_hellman(&PublicKey::from(key));
        let cipher = Aes256::new(shared_secret.to_bytes());
        Ok(cipher)
    }
}

/// Provides client-side cryptography.
pub struct ClientCrypto {
    verifier: ed25519_dalek::PublicKey,
}
impl ClientCrypto {
    /// Creates a ClientCrypto from the peer's signing public key.
    pub fn new(signing_public: &[u8; 32]) -> Result<Self, SigningPublicKeyError> {
        let verifier = ed25519_dalek::PublicKey::from_bytes(signing_public)
            .map_err(|e| SigningPublicKeyError::InvalidData)?;
        Ok(Self { verifier })
    }

    /// Generates a new secret.
    pub fn gen_secret(&self) -> ClientSecret {
        ClientSecret {
            verifier: self.verifier,
            secret: EphemeralSecret::new(&mut OsRng {}),
        }
    }
}
