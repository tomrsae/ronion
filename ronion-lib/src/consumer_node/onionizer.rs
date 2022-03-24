use core::panic;

use aes::Aes256;
use async_std::io::{Cursor, WriteExt};

use crate::protocol::{
    io::{RawOnionReader, RawOnionWriter},
    onion::{Message, Onion, Target},
};

pub struct Onionizer {
    targets: Vec<Target>,
    ciphers: Vec<Aes256>,
}

impl Onionizer {
    pub fn new(targets: Vec<Target>, ciphers: Vec<Aes256>) -> Self {
        Onionizer { targets, ciphers }
    }

    async fn onionize(onion: Onion, cipher: Aes256) -> Vec<u8> {
        let writer = Cursor::new(Vec::<u8>::new());
        let mut onion_writer = RawOnionWriter::new(writer.clone()).with_cipher(cipher);
        onion_writer.write(onion).await.expect("");
        writer.into_inner()
    }

    async fn deonionize(data: Vec<u8>, cipher: Aes256) -> Onion {
        let mut reader = Cursor::new(Vec::<u8>::new());
        let mut onion_reader = RawOnionReader::new(reader.clone()).with_cipher(cipher);
        reader.write(&data);
        onion_reader.read().await.expect("")
    }

    pub async fn grow_onion_relay(&self, payload: Vec<u8>) -> Onion {
        Onionizer::grow_onion(
            Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::Payload(payload),
            },
            self.targets.clone(),
            self.ciphers.clone(),
        )
        .await
    }

    pub async fn grow_onion(mut onion: Onion, targets: Vec<Target>, ciphers: Vec<Aes256>) -> Onion {
        let mut onion_load: Vec<u8>; //At this point targets and ciphers should be of equal length
        for i in 0..targets.len() {
            onion_load = Onionizer::onionize(onion, ciphers[ciphers.len() - 1 - i].clone()).await;
            onion = Onion {
                target: targets[targets.len() - 1 - i].clone(),
                circuit_id: None,
                message: Message::Payload(onion_load),
            };
        }

        onion
    }

    pub async fn peel_onion(onion: Onion, ciphers: Vec<Aes256>) -> Onion {
        let mut out_onion: Onion;
        let mut data = match onion.message {
            Message::Payload(payload) => payload,
            _ => panic!("Got unexpected message type"),
        };

        for i in 0..ciphers.len() - 1 {
            out_onion = Onionizer::deonionize(data, ciphers[i].clone()).await;
            data = match out_onion.message {
                Message::Payload(payload) => payload,
                _ => panic!("Got unexpected message type"),
            };
        }

        Onionizer::deonionize(data, ciphers[ciphers.len() - 1].clone()).await
    }
}
