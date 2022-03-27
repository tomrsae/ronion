use async_std::io::{Cursor, WriteExt};
use async_std::net::{SocketAddr, IpAddr, Ipv4Addr};

use crate::crypto::Aes256;
use crate::protocol::{
    io::{RawOnionReader, RawOnionWriter},
    onion::{Message, Onion, Target},
};

pub struct Onionizer {
    target_ids: Vec<u32>,
    ciphers: Vec<Aes256>,
}

impl Onionizer {
    pub fn new(target_ids: Vec<u32>, ciphers: Vec<Aes256>) -> Self {
        Onionizer {
            target_ids,
            ciphers,
        }
    }

    async fn onionize(onion: Onion, cipher: Aes256) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut onion_writer = RawOnionWriter::new(cursor.get_mut()).with_cipher(cipher);
        onion_writer
            .write(onion)
            .await
            .expect("onionize write failed");
        cursor.into_inner()
    }

    async fn deonionize(data: Vec<u8>, cipher: Aes256) -> Onion {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        cursor.write(&data).await.unwrap();

        cursor.set_position(0);
        let mut onion_reader = RawOnionReader::new(cursor).with_cipher(cipher);

        onion_reader.read().await.expect("onionize read failed")
    }

    pub async fn grow_onion_relay(&self, payload: Vec<u8>, addr: SocketAddr) -> Onion {
        Onionizer::grow_onion(
            Onion {
                circuit_id: None,
                message: Message::Payload(payload),
                target: Target::IP(addr),
            },
            self.target_ids.clone(),
            self.ciphers.clone(),
        )
        .await
    }

    pub async fn peel_onion_relay(&self, onion: Onion) -> Onion {
        Onionizer::peel_onion(onion, self.ciphers.clone()).await
    }

    pub async fn grow_onion(mut onion: Onion, target_ids: Vec<u32>, ciphers: Vec<Aes256>) -> Onion {
        let mut onion_load: Vec<u8>; //At this point targets and ciphers should be of equal length
        for i in 0..target_ids.len() {
            onion_load = Onionizer::onionize(onion, ciphers[ciphers.len() - 1 - i].clone()).await;
            onion = Onion {
                circuit_id: None,
                message: Message::Payload(onion_load),
                target: Target::Relay(target_ids[target_ids.len() - 1 - i].clone()),
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

#[cfg(test)]
mod tests {
    use crate::crypto::{ClientCrypto, ServerCrypto};

    use super::*;

    fn test_cipher() -> Aes256 {
        let server_crypto = ServerCrypto::new();
        let server_sign_key = server_crypto.signing_public();

        let client_crypto = ClientCrypto::new(&server_sign_key).expect("clientcrypto new failed");
        let client_secret = client_crypto.gen_secret();
        let client_public = client_secret.public_key();

        server_crypto.gen_secret().symmetric_cipher(client_public)
    }

    #[async_std::test]
    async fn onionized_can_be_deonionized() {
        let onion = Onion {
            circuit_id: Some(420),
            message: Message::Payload("Naice test guy".as_bytes().to_vec()),
            target: Target::Relay(69),
        };
        let cipher = test_cipher();
        let data = Onionizer::onionize(onion, cipher.clone()).await;
        let actual_onion = Onionizer::deonionize(data, cipher).await;
        assert_eq!(
            Onion {
                circuit_id: Some(420),
                message: Message::Payload("Naice test guy".as_bytes().to_vec()),
                target: Target::Relay(69),
            },
            actual_onion
        );
    }

    #[async_std::test]
    async fn grown_onion_can_be_peeled() {
        let onion = Onion {
            circuit_id: Some(420),
            message: Message::Payload("Naice test guy".as_bytes().to_vec()),
            target: Target::Relay(69),
        };
        let ciphers: Vec<Aes256> = (0..3).into_iter().map(|_| test_cipher()).collect();
        let target_ids = (0..3).collect();
        let grown_onion = Onionizer::grow_onion(onion, target_ids, ciphers.clone()).await;
        let peeled_onion = Onionizer::peel_onion(grown_onion, ciphers).await;

        assert_eq!(
            Onion {
                circuit_id: Some(420),
                message: Message::Payload("Naice test guy".as_bytes().to_vec()),
                target: Target::Relay(69),
            },
            peeled_onion
        )
    }

    #[async_std::test]
    async fn grown_onion_relay_can_be_peeled() {
        let ciphers: Vec<Aes256> = (0..3).into_iter().map(|_| test_cipher()).collect();
        let target_ids = (0..3).collect();
        let onionizer = Onionizer::new(target_ids, ciphers);
        let grown_onion = onionizer
            .grow_onion_relay("Naice test guy".as_bytes().to_vec(), SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))
            .await;
        let peeled_onion = onionizer.peel_onion_relay(grown_onion).await;

        assert_eq!(
            Onion {
                circuit_id: None,
                message: Message::Payload("Naice test guy".as_bytes().to_vec()),
                target: Target::IP(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)),
            },
            peeled_onion
        )
    }
}
