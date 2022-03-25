use aes::Aes256;

use crate::protocol::{io::RawOnionReader, onion::{Onion, Message, Target}};

#[derive(Clone)]
pub struct OnionChannel {
    symmetric_cipher: Aes256,
    //pub stream: TcpStream
}

impl OnionChannel {
    pub fn new(symmetric_cipher: Aes256) -> Self {
        Self {
            symmetric_cipher: symmetric_cipher,
            //stream: stream
        }
    }

    pub fn symmetric_cipher(&self) -> Aes256 {
        self.symmetric_cipher.clone()
    }

    // pub async fn open(&self) -> Result<()> {
    //     let mut reader
    //         = RawOnionReader::new(&self.stream).with_cipher(self.symmetric_cipher());

    //     let onion = reader.read().await?;

    //     match onion.target {
    //         Target::Relay(relay_id) => {
    //             // I am relay node

    //             todo!();
    //         },
    //         Target::IP(ip) => {
    //             // I am exit node
    //             todo!();
    //         },
    //         Target::Current => todo!() // err?
    //     }

    //     if let Message::Payload(payload) = onion.message {
    //         todo!();
    //     } else {
    //         // err?
    //         todo!();
    //     }

    //     Ok(())
    // }
}
