use crate::{
    crypto::Secret,
    protocol::{
        io::{OnionReader, OnionWriter},
        onion::{self, Onion, Target},
    },
};
use aes::Aes256;
use async_std::{
    io::{Cursor, WriteExt},
    net::{IpAddr, TcpStream},
};

pub struct Consumer {
    entry_reader: OnionReader<TcpStream, Aes256>,
    entry_writer: OnionWriter<TcpStream, Aes256>,
    onionizer: Onionizer,
}

impl Consumer {
    pub async fn new(mut n: usize) -> Self {
        let index_pub_key: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];

        let (mut index_reader, index_writer) = Consumer::dial("", index_pub_key).await;

        //index_writer.write(onion) //Write "I want n number of relays to connect to"

        let index_onion = index_reader.read().await.unwrap();
        let (num_relays, mut peer_pub_keys, mut target_ids, entry_ip) =
            Consumer::parse_index_onion(index_onion);
        //check that num_relays match n

        n -= 1;
        let entry_id = target_ids.remove(0);
        let entry_pub_key = peer_pub_keys.remove(0);

        let mut secrets = Secret::create_secrets(n, peer_pub_keys);
        let mut pub_keys = Vec::<[u8; 32]>::with_capacity(n);
        let mut ciphers = Vec::<Aes256>::with_capacity(n);

        for i in 0..n {
            let secret = secrets.remove(i);
            pub_keys.push(secret.gen_pub_key().as_bytes().to_owned());
            ciphers.push(secret.gen_symmetric_cipher());
        }

        let (entry_reader, entry_writer) = Consumer::create_circuit(
            "",
            pub_keys,
            entry_pub_key,
            target_ids.clone(),
            ciphers.clone(),
        )
        .await;

        Consumer {
            entry_reader,
            entry_writer,
            onionizer: Onionizer::new(target_ids, ciphers), //Both target ids and ips?
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
        let secret = Secret::new(peer_pub_key);
        let pub_key = secret.gen_pub_key();

        stream.write(&pub_key.to_bytes()).await;

        let cipher = secret.gen_symmetric_cipher();
        (
            OnionReader::new(stream.clone(), cipher.clone()),
            OnionWriter::new(stream.clone(), cipher.clone()),
        )
    }

    fn parse_index_onion(onion: Onion) -> (usize, Vec<[u8; 32]>, Vec<Target>, IpAddr) {
        let n: usize = 0;
        let keys = Vec::<[u8; 32]>::new();
        let ids = Vec::<Target>::new();
        let ip: IpAddr;
        //onion.payload.chunks_exact(chunk_size).map(f)....
        panic!("data format not yet implemented");

        (n, keys, ids, ip)
    }

    pub async fn send_message(&mut self, payload: Vec<u8>) -> () {
        let onion = self.onionizer.grow_onion_relay(payload).await;
        self.entry_writer.write(onion).await.unwrap();
    }

    pub async fn recv_message(&mut self) -> Vec<u8> {
        let onion = self.entry_reader.read().await.unwrap();
        //Check target?? (probably unneccesary)
        onion.payload
    }

    async fn create_circuit(
        addr: &str,
        pub_keys: Vec<[u8; 32]>,
        entry_pub_key: [u8; 32],
        targets: Vec<Target>, //targets[0] should always be Target::Current -> always the onion core
        ciphers: Vec<Aes256>,
    ) -> (
        OnionReader<TcpStream, Aes256>,
        OnionWriter<TcpStream, Aes256>,
    ) {
        let (entry_reader, mut entry_writer) = Consumer::dial("", entry_pub_key).await;

        for i in 0..targets.len() {
            let onion = Onionizer::grow_onion(
                targets[0..i + 1].to_vec(), //Should send copy
                ciphers[0..i + 1].to_vec(), //Should send copy
                pub_keys[i].to_vec(),
            )
            .await;
            let res = entry_writer.write(onion);
        }

        (entry_reader, entry_writer)
    }
}

pub struct Onionizer {
    targets: Vec<Target>,
    ciphers: Vec<Aes256>,
}

impl Onionizer {
    pub fn new(targets: Vec<Target>, ciphers: Vec<Aes256>) -> Self {
        Onionizer { targets, ciphers }
    }

    async fn onionize(target: Target, payload: Vec<u8>, cipher: Aes256) -> Vec<u8> {
        let writer = Cursor::new(Vec::<u8>::new());
        let onion = Onion { target, payload };
        let mut onion_writer = OnionWriter::new(writer.clone(), cipher);

        onion_writer.write(onion).await.expect("");
        writer.into_inner()
    }

    pub async fn grow_onion_relay(&self, payload: Vec<u8>) -> Onion {
        Onionizer::grow_onion(self.targets.clone(), self.ciphers.clone(), payload).await
    }

    pub async fn grow_onion(
        mut targets: Vec<Target>,
        mut ciphers: Vec<Aes256>,
        payload: Vec<u8>,
    ) -> Onion {
        if targets.len() == 1 {
            return Onion {
                target: targets[0].clone(),
                payload,
            };
        }

        let mut onion_load = Onionizer::onionize(
            targets.remove(targets.len() - 1),
            payload,
            ciphers.remove(ciphers.len() - 1),
        )
        .await;

        for i in 0..targets.len() - 1 {
            onion_load = Onionizer::onionize(
                targets[targets.len() - 1 - i].clone(),
                onion_load,
                ciphers[ciphers.len() - 1 - i].clone(),
            )
            .await
        }

        Onion {
            target: targets[0].clone(),
            payload: onion_load,
        }
    }
}
