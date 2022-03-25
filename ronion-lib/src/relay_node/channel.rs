use aes::Aes256;
use async_std::net::TcpStream;

#[derive(Clone)]
pub struct Channel {
    symmetric_cipher: Aes256,
    pub stream: TcpStream
}

impl Channel {
    pub fn new(stream: TcpStream, symmetric_cipher: Aes256) -> Self {
        Self {
            symmetric_cipher: symmetric_cipher,
            stream: stream
        }
    }

    pub fn symmetric_cipher(&self) -> Aes256 {
        self.symmetric_cipher.clone()
    }

    pub async fn open(&self) {

    }
}
