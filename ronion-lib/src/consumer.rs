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
        let mut peer_pub_keys: Vec<[u8; 96]>;
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

        let pls_remove_me = [0u8; 32];
        let mut cryptos: Vec<ClientSecret> = (0..n)
            .into_iter()
            .map(|_| ClientCrypto::new(&pls_remove_me).unwrap().gen_secret())
            .collect();
        let mut pub_keys = Vec::<[u8; 32]>::with_capacity(n);
        let mut ciphers = Vec::<Aes256>::with_capacity(n);

        let mut secrets = for i in 0..n {
            let crypto = cryptos.remove(i);
            pub_keys.push(crypto.public_key().to_owned());
            ciphers.push(crypto.symmetric_cipher(peer_pub_keys[i]).unwrap());
        };

        let (entry_reader, entry_writer, ciphers) = Consumer::create_circuit(
            &entry_ip.to_string(),
            pub_keys,
            [0u8; 32],
            target_ids.clone(),
            circuit_id,
            ciphers.clone(),
        )
        .await;

        Consumer {
            entry_reader,
            entry_writer,
            onionizer: Onionizer::new(target_ids, circuit_id, ciphers),
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

        let peer_pub_key = match hello_resp.message {
            Message::HelloResponse(payload) => payload,
            _ => {
                panic!("Did not get 'HelloResponse'")
            }
        };

        let cipher = secret.symmetric_cipher(peer_pub_key).unwrap();
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
                Some(v) => todo!(),
                None => todo!(),
            },
            _ => panic!("Got unexpected message"),
        }
    }

    async fn create_circuit(
        addr: &str,
        pub_keys: Vec<[u8; 32]>,
        entry_pub_key: [u8; 32],
        targets: Vec<Target>, //targets[0] should always be Target::Current -> always the onion core
        circuit_id: Option<u32>,
    ) -> (
        OnionReader<TcpStream, Aes256>,
        OnionWriter<TcpStream, Aes256>,
        Vec<Aes256>,
    ) {
        let (mut entry_reader, mut entry_writer) = Consumer::dial(addr, entry_pub_key).await;
        let mut ciphers = Vec::<Aes256>::new();
        let mut onion: Onion;
        for i in 0..targets.len() {
            onion = Onionizer::grow_circuit_onion(
                targets[0..i + 1].to_vec(), //Should send copy
                circuit_id,
                &mut ciphers,
                pub_keys[i],
            )
            .await;
            entry_writer.write(onion).await;
            onion = entry_reader.read().await.unwrap();
            onion = Onionizer::peel_circuit_onion(onion).await;
        }

        (entry_reader, entry_writer, ciphers)
    }
}

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

    async fn serialize_onion(onion: Onion, cipher: Aes256) -> Vec<u8> {
        let writer = Cursor::new(Vec::<u8>::new());
        let mut onion_writer = RawOnionWriter::new(writer.clone()).with_cipher(cipher);
        onion_writer.write(onion).await.expect("");
        writer.into_inner()
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

    pub async fn grow_circuit_onion(
        mut targets: Vec<Target>,
        circuit_id: Option<u32>,
        ciphers: &mut Vec<Aes256>, //should be one less than targets
        pub_key: [u8; 32],
    ) -> Onion {
        //Core is the newest value added to the vectors. It should be the hellorequest
        let mut onion = Onion {
            target: targets.remove(targets.len() - 1),
            circuit_id,
            message: Message::HelloRequest(pub_key),
        };

        let mut onion_load: Vec<u8>;
        for i in 0..targets.len() - 1 {
            onion_load =
                Onionizer::serialize_onion(onion, ciphers[ciphers.len() - 1].clone()).await;
            onion = Onion {
                target: targets[targets.len() - 1 - i].clone(),
                circuit_id,
                message: Message::Payload(onion_load),
            };
        }

        onion
    }

    pub async fn peel_circuit_onion(onion: Onion) -> Onion {}

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
}
