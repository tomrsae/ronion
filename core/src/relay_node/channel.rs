use std::net::SocketAddr;

use aes::Aes256;
use async_std::{net::TcpStream, io::{Result, Read, Write}};

use crate::{protocol::{io::{OnionReader, OnionWriter, RawOnionReader, RawOnionWriter}, onion::{Onion, Message, Target, HelloRequest, ClientType}}, crypto::ClientSecret};

pub struct OnionChannel {
    symmetric_cipher: Aes256,
    reader: OnionReader<TcpStream, Aes256>,
    writer: OnionWriter<TcpStream, Aes256>,
    peer_addr: SocketAddr
}

impl OnionChannel {
    pub fn new(stream: TcpStream, symmetric_cipher: Aes256) -> Self {
        Self {
            symmetric_cipher: symmetric_cipher,
            reader: RawOnionReader::new(stream).with_cipher(symmetric_cipher),
            writer: RawOnionWriter::new(stream).with_cipher(symmetric_cipher),
            peer_addr: stream.peer_addr().expect("Failed to retrieve peer address")
        }
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    pub fn symmetric_cipher(&self) -> Aes256 {
        self.symmetric_cipher.clone()
    }

    pub async fn recv_payload(&mut self) -> Result<Onion> {
        self.reader.read().await
    }

    pub async fn send_payload(&mut self, onion: Onion) -> Result<()> {
        self.writer.write(onion).await
    }

    pub async fn reach_relay(stream: TcpStream, secret: ClientSecret) -> Result<OnionChannel> {
        let (reader, writer)
            = &mut (RawOnionReader::new(&stream), RawOnionWriter::new(&stream));

        let pub_key = secret.public_key();
    
        let mut writer = RawOnionWriter::new(&stream);
        writer.write(
            Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::HelloRequest(HelloRequest { client_type: ClientType::Relay, public_key: pub_key })
            }
        ).await?;

        let mut reader = RawOnionReader::new(&stream);
        let hello_response = reader.read().await?;

        let symmetric_cipher
            = if let Message::HelloResponse(peer_key) = hello_response.message {
                secret.symmetric_cipher(peer_key).expect("Failed to create symmetric cipher")
            } else {
                //err?
                todo!()
            }

        Ok(OnionChannel::new(stream, symmetric_cipher))
    }

    // pub async fn open(&self) -> Result<()> {
    //     let mut reader
    //         = RawOnionReader::new(&self.stream).with_cipher(self.symmetric_cipher());

    //     let onion = reader.read().await?;

    //     match onion.target {
    //         Target::Relay(relay_id) => {
    //             // I am relay node

    //             todo!();
    //         },
    //         Target::IP(ip) => {
    //             // I am exit node
    //             todo!();
    //         },
    //         Target::Current => todo!() // err?
    //     }

    //     if let Message::Payload(payload) = onion.message {
    //         todo!();
    //     } else {
    //         // err?
    //         todo!();
    //     }

    //     Ok(())
    // }
}
