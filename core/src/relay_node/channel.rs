use std::io::Result;

use aes::Aes256;
use async_std::net::TcpStream;

use crate::protocol::{io::RawOnionReader, onion::{Onion, Message}};

#[derive(Clone)]
pub struct OnionChannel {
    symmetric_cipher: Aes256,
    pub stream: TcpStream
}

impl OnionChannel {
    pub fn new(stream: TcpStream, symmetric_cipher: Aes256) -> Self {
        Self {
            symmetric_cipher: symmetric_cipher,
            stream: stream
        }
    }

    pub fn symmetric_cipher(&self) -> Aes256 {
        self.symmetric_cipher.clone()
    }

    pub async fn open(&self) -> Result<()> {
        let mut reader
            = RawOnionReader::new(&self.stream).with_cipher(self.symmetric_cipher());

        let onion = reader.read().await?;
        if let Message::Payload(payload) = onion.message {
            self.handle_payload(payload).await;
        } else {
            // err?
        }

        Ok(())
    }

    async fn handle_payload(&self, payload: Vec<u8>) {

    }
}
