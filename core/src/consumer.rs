use crate::{
    crypto::Secret,
    protocol::{
        io::{OnionReader, OnionWriter},
        onion::{Onion, Target},
    },
};
use aes::Aes256;
use async_std::{io::Cursor, net::TcpStream};

pub struct Consumer {
    public_keys: Vec<[u8; 32]>,
    reader: OnionReader<TcpStream, Aes256>,
    writer: OnionWriter<TcpStream, Aes256>,
    onionizer: Onionizer,
}

impl Consumer {
    fn new(
        n: usize,
        stream: &TcpStream,
        peer_public_keys: Vec<[u8; 32]>,
        targets: Vec<Target>,
    ) -> Self {
        let mut secrets = Secret::create_secrets(n, peer_public_keys);
        let mut reader: OnionReader<TcpStream, Aes256>;
        let mut public_keys = Vec::<[u8; 32]>::with_capacity(n);
        let mut ciphers = Vec::<Aes256>::with_capacity(n);

        for i in 0..(n - 1) {
            let secret = secrets.remove(i);
            public_keys.push(secret.gen_pub_key().as_bytes().to_owned());
            ciphers.push(secret.gen_symmetric_cipher());
        }

        Consumer {
            public_keys,
            reader: OnionReader::new(stream.clone(), ciphers[n - 1].clone()),
            writer: OnionWriter::new(stream.clone(), ciphers[n - 1].clone()),
            onionizer: Onionizer::new(targets, ciphers.clone()),
        }
    }

    pub async fn dial_index() -> TcpStream {
        let addr = ""; //Decide addresses to use/how to find address?
        TcpStream::connect(addr).await.expect("")
    }

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
