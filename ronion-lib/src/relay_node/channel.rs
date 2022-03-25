use aes::Aes256;

#[derive(Clone)]
pub struct Channel {
    symmetric_cipher: Aes256
}

impl Channel {
    pub fn new(symmetric_cipher: Aes256) -> Self {
        Self {
            symmetric_cipher: symmetric_cipher
        }
    }

    pub fn symmetric_cipher(&self) -> Aes256 {
        self.symmetric_cipher.clone()
    }

    pub async fn open(&self) {
        
    }
}
