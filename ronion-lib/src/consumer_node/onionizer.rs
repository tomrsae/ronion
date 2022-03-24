use core::panic;

use aes::Aes256;
use async_std::io::{Cursor, WriteExt};

use crate::protocol::{
    io::{RawOnionReader, RawOnionWriter},
    onion::{Message, Onion, Target},
};

pub struct Onionizer {
    targets: Vec<Target>,
    circuit_id: Option<u32>,
    ciphers: Vec<Aes256>,
}

impl Onionizer {
    pub fn new(targets: Vec<Target>, circuit_id: Option<u32>, ciphers: Vec<Aes256>) -> Self {
        Onionizer {
            targets,
            circuit_id,
            ciphers,
        }
    }

    async fn onionize(
        target: Target,
        circuit_id: Option<u32>,
        payload: Vec<u8>,
        cipher: Aes256,
    ) -> Vec<u8> {
        let writer = Cursor::new(Vec::<u8>::new());
        let onion = Onion {
            target,
            circuit_id,
            message: Message::Payload(payload),
        };
        let mut onion_writer = RawOnionWriter::new(writer.clone()).with_cipher(cipher);

        onion_writer.write(onion).await.expect("");
        writer.into_inner()
    }

    pub async fn grow_onion_relay(&self, payload: Vec<u8>) -> Onion {
        Onionizer::grow_onion(
            self.targets.clone(),
            self.circuit_id,
            self.ciphers.clone(),
            payload,
        )
        .await
    }

    pub async fn grow_onion(
        mut targets: Vec<Target>,
        circuit_id: Option<u32>,
        mut ciphers: Vec<Aes256>,
        payload: Vec<u8>,
    ) -> Onion {
        if targets.len() == 1 {
            return Onion {
                target: targets[0].clone(),
                circuit_id,
                message: Message::Payload(payload),
            };
        }

        //Core is the newest value added to the vectors
        let mut onion_load = Onionizer::onionize(
            targets.remove(targets.len() - 1),
            circuit_id,
            payload,
            ciphers.remove(ciphers.len() - 1),
        )
        .await;

        for i in 0..targets.len() - 1 {
            onion_load = Onionizer::onionize(
                targets[targets.len() - 1 - i].clone(), //Could use remove here insted of clone?
                circuit_id,
                onion_load,
                ciphers[ciphers.len() - 1 - i].clone(), //Could use remove here insted of clone?
            )
            .await
        }

        Onion {
            target: targets[0].clone(),
            circuit_id,
            message: Message::Payload(onion_load),
        }
    }

    async fn serialize_onion(onion: Onion, cipher: Aes256) -> Vec<u8> {
        let writer = Cursor::new(Vec::<u8>::new());
        let mut onion_writer = RawOnionWriter::new(writer.clone()).with_cipher(cipher);
        onion_writer.write(onion).await.expect("");
        writer.into_inner()
    }

    async fn unserialize_onion(data: Vec<u8>, cipher: Aes256) -> Onion {
        let mut reader = Cursor::new(Vec::<u8>::new());
        let mut onion_reader = RawOnionReader::new(reader.clone()).with_cipher(cipher);
        reader.write(&data);
        onion_reader.read().await.expect("")
    }

    pub async fn grow_circuit_onion(
        mut targets: Vec<Target>,
        ciphers: &mut Vec<Aes256>, //should be one less than targets
        payload: [u8; 32],
    ) -> Onion {
        //Core is the newest value added to the vectors. It should be the hellorequest
        let mut onion = Onion {
            target: targets.remove(targets.len() - 1),
            circuit_id: None,
            message: Message::HelloRequest(payload),
        };

        let mut onion_load: Vec<u8>; //At this point targets and ciphers should be of equal length
        for i in 0..targets.len() - 1 {
            onion_load =
                Onionizer::serialize_onion(onion, ciphers[ciphers.len() - 1 - i].clone()).await;
            onion = Onion {
                target: targets[targets.len() - 1 - i].clone(),
                circuit_id: None,
                message: Message::Payload(onion_load),
            };
        }

        onion
    }

    pub async fn peel_circuit_onion(onion: Onion, ciphers: &mut Vec<Aes256>) -> Onion {
        let mut out_onion: Onion;
        let mut data = match onion.message {
            Message::Payload(payload) => payload,
            _ => panic!("Got unexpected message type"),
        };

        for i in 0..ciphers.len() - 1 {
            out_onion = Onionizer::unserialize_onion(data, ciphers[i].clone()).await;
            data = match out_onion.message {
                Message::Payload(payload) => payload,
                _ => panic!("Got unexpected message type"),
            };
        }

        Onionizer::unserialize_onion(data, ciphers[ciphers.len() - 1].clone()).await
    }
}
