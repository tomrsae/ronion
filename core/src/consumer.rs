use crate::{
    crypto::Secret,
    protocol::{
        io::{OnionReader, OnionWriter},
        onion::{Onion, Target},
    },
};
use aes::Aes256;
use async_std::{
    io::{Cursor, ReadExt, WriteExt},
    net::TcpStream,
};

pub struct Consumer {
    public_keys: Vec<[u8; 32]>,
    reader: OnionReader<TcpStream, Aes256>,
    writer: OnionWriter<TcpStream, Aes256>,
    onionizer: Onionizer,
    entry_target: Target,
}

impl Consumer {
    fn new(
        n: usize,
        stream: &TcpStream,
        peer_public_keys: Vec<[u8; 32]>,
        mut targets: Vec<Target>,
    ) -> Self {
        let index_pub_key: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let mut secrets = Secret::create_secrets(n, peer_public_keys);
        let mut public_keys = Vec::<[u8; 32]>::with_capacity(n);
        let mut reader: OnionReader<TcpStream, Aes256>;
        let mut ciphers = Vec::<Aes256>::with_capacity(n);

        for i in 0..n {
            let secret = secrets.remove(i);
            public_keys.push(secret.gen_pub_key().as_bytes().to_owned());
            ciphers.push(secret.gen_symmetric_cipher());
        }
        let entry_target = targets.remove(n - 1);

        Consumer {
            public_keys,
            reader: OnionReader::new(stream.clone(), ciphers[n - 1].clone()),
            writer: OnionWriter::new(stream.clone(), ciphers[n - 1].clone()),
            onionizer: Onionizer::new(targets, ciphers.clone()),
            entry_target,
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

    // pub async fn handshake_entry(
    //     stream: &mut TcpStream,
    //     entry_target: Target,
    //     entry_pub_key: [u8; 32],
    // ) {
    //     let secret = Secret::new(index_pub_key);
    //     let cipher = secret.gen_symmetric_cipher();
    // }

    //Fix return
    pub fn send_message() -> bool {
        true
    }

    //Fix return
    pub fn recv_message() -> bool {
        true
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

    pub async fn grow_onion(&self, num_layers: usize, payload: Vec<u8>) -> Vec<u8> {
        if num_layers <= 0 {
            panic!("Cannot layer onion with 'nothing'")
        }
        let targets = self.targets.clone();
        let ciphers = self.ciphers.clone();
        let mut onion_load =
            Onionizer::onionize(targets[0].clone(), payload, ciphers[0].clone()).await;

        for i in 1..num_layers {
            onion_load =
                Onionizer::onionize(targets[i].clone(), onion_load, ciphers[i].clone()).await
        }
        onion_load
    }
}
