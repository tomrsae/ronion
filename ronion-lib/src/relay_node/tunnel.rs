use aes::Aes256;

pub struct Tunnel {
    pub peer_key: [u8; 32],
    pub symmetric_cipher: Aes256,
    pub public_key: [u8; 96]
}