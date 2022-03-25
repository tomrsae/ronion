use aes::Aes256;

#[derive(Clone)]
pub struct Circuit {
    pub id: u32,
    pub peer_key: [u8; 32],
    pub symmetric_cipher: Aes256,
    pub public_key: [u8; 96]
}

impl Circuit {
    pub fn symmetric_cipher(&self) -> Aes256 {
        self.symmetric_cipher.clone()
    }
}