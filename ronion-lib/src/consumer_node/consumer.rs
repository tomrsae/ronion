use std::net::SocketAddr;

use crate::{
    crypto::{ClientCrypto, ClientSecret},
    protocol::{
        io::{OnionReader, OnionWriter, RawOnionReader, RawOnionWriter},
        onion::{Message, Onion, Target},
    },
};
use aes::Aes256;
use async_std::net::TcpStream;

use super::onionizer::Onionizer;

pub struct Consumer {
    entry_reader: OnionReader<TcpStream, Aes256>,
    entry_writer: OnionWriter<TcpStream, Aes256>,
    onionizer: Onionizer,
}

impl Consumer {
    pub async fn new(index_addr: &str, index_pub_key: [u8; 32]) -> Self {
        let (mut index_reader, mut index_writer) = Consumer::dial(index_addr, index_pub_key).await;

        index_writer
            .write(Onion {
                circuit_id: None,
                message: Message::GetRelaysRequest(),
                target: Target::Current,
            })
            .await
            .expect("index writer failed");

        let index_onion = index_reader.read().await.expect("index reader failed");
        let relays = match index_onion.message {
            Message::GetRelaysResponse(relays) => relays,
            _ => panic!("Got unexpected message"),
        };

        let mut target_ids = Vec::<u32>::with_capacity(relays.len());
        let mut target_ips = Vec::<SocketAddr>::with_capacity(relays.len());
        let mut peer_pub_keys = Vec::<[u8; 32]>::with_capacity(relays.len());

        for relay in relays {
            target_ids.push(relay.id);
            target_ips.push(relay.addr);
            peer_pub_keys.push(relay.pub_key);
        }

        //In general the higher the index in the vectors, the closer the value is to the onion core
        //This means targets[targets.len() -1] is the core, and targets[0] is always the outermost layer

        let (entry_reader, entry_writer, ciphers) =
            Consumer::create_circuit(target_ids.clone(), target_ips, peer_pub_keys).await;

        Consumer {
            entry_reader,
            entry_writer,
            onionizer: Onionizer::new(target_ids, ciphers),
        }
    }

    pub async fn dial(
        addr: &str,
        peer_pub_key: [u8; 32],
    ) -> (
        OnionReader<TcpStream, Aes256>,
        OnionWriter<TcpStream, Aes256>,
    ) {
        let mut stream = TcpStream::connect(addr).await.expect("");
        Consumer::handshake(&mut stream, peer_pub_key).await
    }

    pub async fn handshake(
        stream: &mut TcpStream,
        peer_pub_key: [u8; 32],
    ) -> (
        OnionReader<TcpStream, Aes256>,
        OnionWriter<TcpStream, Aes256>,
    ) {
        let client_crypto = match ClientCrypto::new(&peer_pub_key) {
            Ok(v) => v,
            Err(e) => panic!("could not create crypto client: {:?}", e),
        };
        let secret = client_crypto.gen_secret();
        let pub_key = secret.public_key();

        let mut raw_writer = RawOnionWriter::new(stream.clone());
        let mut raw_reader = RawOnionReader::new(stream.clone());

        raw_writer
            .write(Onion {
                circuit_id: None,
                message: Message::HelloRequest(pub_key),
                target: Target::Current,
            })
            .await
            .expect("raw handshake writer failed");
        let hello_resp = raw_reader.read().await.expect("raw reader failed");

        let signed_public_key = match hello_resp.message {
            Message::HelloResponse(signed_public_key) => signed_public_key,
            _ => {
                panic!("Did not get 'HelloResponse'")
            }
        };

        let cipher = secret
            .symmetric_cipher(signed_public_key)
            .expect("symmetric cipher gen failed");
        (
            raw_reader.with_cipher(cipher.clone()),
            raw_writer.with_cipher(cipher.clone()),
        )
    }

    pub async fn send_message(&mut self, payload: Vec<u8>) -> () {
        let onion = self.onionizer.grow_onion_relay(payload).await;
        self.entry_writer
            .write(onion)
            .await
            .expect("entry writer failed");
    }

    pub async fn recv_message(&mut self) -> Vec<u8> {
        let onion = self.entry_reader.read().await.expect("entry reader failed");
        let peeled_onion = self.onionizer.peel_onion_relay(onion).await;
        match peeled_onion.message {
            Message::Payload(load) => load,
            Message::Close(msg) => match msg {
                Some(_v) => todo!(),
                None => todo!(),
            },
            _ => panic!("Got unexpected message"),
        }
    }

    //Per iteration:
    // - Create clientcrypto(peer_public) and client secret.
    // - Grow circuit onion with HelloRequest(secret.public_key).
    // - Send and recieve onion
    // - Peel onion and match Message type. If true, create and add cipher
    //   to vector of ciphers.
    // - Return entry_reader, entry_writer and ciphers. Circuit is done
    async fn create_circuit(
        mut target_ids: Vec<u32>,
        mut target_ips: Vec<SocketAddr>,
        mut peer_keys: Vec<[u8; 32]>,
    ) -> (
        OnionReader<TcpStream, Aes256>,
        OnionWriter<TcpStream, Aes256>,
        Vec<Aes256>,
    ) {
        let mut crypto: ClientCrypto;
        let mut secret: ClientSecret;
        let mut secret_public: [u8; 32];
        let mut ciphers = Vec::<Aes256>::new();
        let mut onion: Onion;

        // Decrement all lists for the first onion tunnel (entry node)
        target_ids.remove(target_ids.len() - 1);
        let (mut entry_reader, mut entry_writer) = Consumer::dial(
            &target_ips.remove(target_ips.len() - 1).to_string(),
            peer_keys.remove(peer_keys.len() - 1),
        )
        .await;

        if !(target_ids.len() == target_ips.len() && target_ips.len() == peer_keys.len()) {
            panic!("All lists must be of equal length")
        }

        for i in 0..target_ids.len() {
            crypto = ClientCrypto::new(&peer_keys.remove(peer_keys.len() - 1))
                .expect("clientcrypto new failed");
            secret = crypto.gen_secret();
            secret_public = secret.public_key();
            onion = Onionizer::grow_onion(
                Onion {
                    circuit_id: None,
                    message: Message::HelloRequest(secret_public),
                    target: Target::Relay(target_ids[i].clone()),
                },
                target_ids[0..i].to_vec(), //Should send copy
                ciphers[0..i].to_vec(),    //Empty first time
            )
            .await;
            match entry_writer.write(onion).await {
                Ok(v) => v,
                Err(_e) => panic!("Write error"),
            };
            onion = entry_reader.read().await.expect("entry read failed");
            onion = Onionizer::peel_onion(onion, ciphers.clone()).await;
            ciphers.push(match onion.message {
                Message::HelloResponse(signed_public_key) => secret
                    .symmetric_cipher(signed_public_key)
                    .expect("symmetric cipher gen failed"),
                _ => panic!("Got unexpected Message type"),
            })
        }

        (entry_reader, entry_writer, ciphers)
    }
}
