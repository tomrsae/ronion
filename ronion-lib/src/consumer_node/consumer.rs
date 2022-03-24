use crate::{
    crypto::{ClientCrypto, ClientSecret},
    protocol::{
        io::{serialize_relays, OnionReader, OnionWriter, RawOnionReader, RawOnionWriter},
        onion::{self, Message, Onion, Target},
    },
};
use aes::Aes256;
use async_std::{
    fs::read,
    io::{Cursor, ReadExt, WriteExt},
    net::{IpAddr, TcpStream},
};

use super::onionizer::Onionizer;

pub struct Consumer {
    entry_reader: OnionReader<TcpStream, Aes256>,
    entry_writer: OnionWriter<TcpStream, Aes256>,
    onionizer: Onionizer,
}

impl Consumer {
    pub async fn new(mut n: usize, index_pub_key: [u8; 32], index_addr: &str) -> Self {
        let (mut index_reader, index_writer) = Consumer::dial(index_addr, index_pub_key).await;

        //index_writer.write(onion) //Write "I want n number of relays to connect to"

        let index_onion = index_reader.read().await.unwrap();
        let num_relays: usize;
        let mut peer_pub_keys: Vec<[u8; 32]>;
        let mut target_ids: Vec<Target> = todo!();
        let entry_ip: IpAddr;
        let circuit_id = Some(2);
        //Consumer::parse_index_onion(index_onion);
        //check that num_relays match n

        //In general the higher the index in the vectors, the closer the value is to the onion core
        //This means targets[targets.len() -1] is the core, and targets[0] is always the outermost layer

        n -= 1;
        target_ids.remove(0);
        let entry_pub_key = peer_pub_keys.remove(0);

        let (entry_reader, entry_writer, ciphers) =
            Consumer::create_circuit(&entry_ip.to_string(), peer_pub_keys, target_ids.clone())
                .await;

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
                target: Target::Current,
                circuit_id: None,
                message: Message::HelloRequest(pub_key),
            })
            .await;
        let hello_resp = raw_reader.read().await.unwrap();

        let signed_public_key = match hello_resp.message {
            Message::HelloResponse(signed_public_key) => signed_public_key,
            _ => {
                panic!("Did not get 'HelloResponse'")
            }
        };

        let cipher = secret.symmetric_cipher(signed_public_key).unwrap();
        (
            raw_reader.with_cipher(cipher.clone()),
            raw_writer.with_cipher(cipher.clone()),
        )
    }

    pub async fn send_message(&mut self, payload: Vec<u8>) -> () {
        let onion = self.onionizer.grow_onion_relay(payload).await;
        self.entry_writer.write(onion).await.unwrap();
    }

    pub async fn recv_message(&mut self) -> Vec<u8> {
        let onion = self.entry_reader.read().await.unwrap();
        //Check target?? (probably unneccesary)
        match onion.message {
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
        addr: &str,
        mut peer_keys: Vec<[u8; 32]>,
        targets: Vec<Target>, //targets[0] should always be Target::Current -> always the onion core
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

        let (mut entry_reader, mut entry_writer) =
            Consumer::dial(addr, peer_keys.remove(peer_keys.len() - 1)).await;

        for i in 0..targets.len() {
            crypto = ClientCrypto::new(&peer_keys.remove(peer_keys.len() - 1)).unwrap();
            secret = crypto.gen_secret();
            secret_public = secret.public_key();
            onion = Onionizer::grow_onion(
                Onion {
                    target: targets[i].clone(),
                    circuit_id: None,
                    message: Message::HelloRequest(secret_public),
                },
                targets[0..i].to_vec(), //Should send copy
                ciphers[0..i].to_vec(), //Empty first time
            )
            .await;
            match entry_writer.write(onion).await {
                Ok(v) => v,
                Err(_e) => panic!("Write error"),
            };
            onion = entry_reader.read().await.unwrap();
            onion = Onionizer::peel_onion(onion, ciphers.clone()).await;
            ciphers.push(match onion.message {
                Message::HelloResponse(signed_public_key) => {
                    secret.symmetric_cipher(signed_public_key).unwrap()
                }
                _ => panic!("Got unexpected Message type"),
            })
        }

        (entry_reader, entry_writer, ciphers)
    }
}
