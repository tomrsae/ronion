use crate::{
    crypto::{Aes256, ClientCrypto, ClientSecret},
    protocol::{
        io::{OnionReader, OnionWriter, RawOnionReader, RawOnionWriter},
        onion::{ClientType, HelloRequest, Message, Onion, Relay, Target},
    },
};
use async_std::net::{SocketAddr, TcpStream};

use super::onionizer::Onionizer;

pub struct Consumer {
    entry_reader: OnionReader<TcpStream, Aes256>,
    entry_writer: OnionWriter<TcpStream, Aes256>,
    onionizer: Onionizer,
}

impl Consumer {
    // Creates a new Consumer instance by dialing an index_addr with
    // an index key. After receiving relays it will attemtp to set up
    // its overral circuit in the network.
    pub async fn new(index_addr: String, index_pub_key: [u8; 32]) -> Self {
        let (mut index_reader, mut index_writer) =
            Consumer::dial_with_key(index_addr, index_pub_key).await;

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

        println!("In consumer new before circuit creation");

        let (entry_reader, entry_writer, target_ids, ciphers) =
            Consumer::create_circuit(relays).await;

        println!("In consumer new after circuit creation");

        Consumer {
            entry_reader,
            entry_writer,
            onionizer: Onionizer::new(target_ids, ciphers),
        }
    }

    // Sets upp a tcp connectioon to the given addr.
    async fn dial(addr: String) -> TcpStream {
        println!("{:?}: ", addr);
        TcpStream::connect(addr).await.expect("unable to connect")
    }

    // Dials, given a key. It uses said key to execute a handshake with the recieveing
    // node at the specified addr.
    async fn dial_with_key(
        addr: String,
        peer_pub_key: [u8; 32],
    ) -> (
        OnionReader<TcpStream, Aes256>,
        OnionWriter<TcpStream, Aes256>,
    ) {
        Consumer::handshake(&mut Consumer::dial(addr).await, peer_pub_key).await
    }

    // Attempts to create a ronion handshake with the given stream. From the handshake
    // we will end up with a OnionReader and OnionWriter with the same cipher.
    async fn handshake(
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
                message: Message::HelloRequest(HelloRequest {
                    client_type: ClientType::Consumer,
                    public_key: pub_key,
                }),
                target: Target::Current,
            })
            .await
            .expect("raw handshake writer failed");
        let hello_resp = raw_reader.read().await.expect("raw reader failed");

        let signed_public_key = match hello_resp.message {
            Message::HelloResponse(signed_public_key) => signed_public_key,
            _ => {
                panic!("expected 'HelloResponse', got {:?}", hello_resp.message)
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

    // Method to be called by other implementations utelising consumer. Sends
    // the specified payload as an onion across the consumer's circuit.
    pub async fn send_message(&mut self, payload: Vec<u8>, addr: SocketAddr) -> () {
        let onion = self.onionizer.grow_onion_relay(payload, addr).await;
        self.entry_writer.write(onion).await.unwrap();
    }

    // Method to be called by other implementations utelising consumer. Recieves
    // a payload from an onion recieved over the consumer's circuit.
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

    //Creates the network circuit before actually utelizing the network.
    async fn create_circuit(
        mut relays: Vec<Relay>,
    ) -> (
        OnionReader<TcpStream, Aes256>,
        OnionWriter<TcpStream, Aes256>,
        Vec<u32>,
        Vec<Aes256>,
    ) {
        let mut crypto: ClientCrypto;
        let mut secret: ClientSecret;
        let mut secret_public: [u8; 32];
        let mut ciphers = Vec::<Aes256>::new();
        let mut onion: Onion;

        if relays.len() == 0 {
            panic!("Relays cannot be zero in length")
        }
        println!("Relays: {:?}", relays);
        let entry_node = relays.remove(relays.len() - 1);
        let (mut entry_reader, mut entry_writer) =
            Consumer::dial_with_key(entry_node.addr.to_string(), entry_node.pub_key).await;

        for i in 0..relays.len() {
            crypto = ClientCrypto::new(&relays.clone()[relays.len() - 1].pub_key)
                .expect("clientcrypto new failed");
            secret = crypto.gen_secret();
            secret_public = secret.public_key();
            onion = Onionizer::grow_onion(
                Onion {
                    circuit_id: None,
                    message: Message::HelloRequest(HelloRequest {
                        client_type: ClientType::Consumer,
                        public_key: secret_public,
                    }),
                    target: Target::Relay(relays.clone()[i].id),
                },
                relays.clone()[0..i]
                    .into_iter()
                    .map(|relay| relay.id)
                    .collect(),
                ciphers[0..i].to_vec(),
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

        let target_ids = relays.into_iter().map(|relay| relay.id).collect();

        (entry_reader, entry_writer, target_ids, ciphers)
    }
}
