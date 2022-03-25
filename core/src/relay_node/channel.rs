use std::{net::SocketAddr, cell::RefCell, sync::Arc};

use aes::Aes256;
use async_std::{net::TcpStream, io::{Result, Read, Write, Cursor}, sync::Mutex};

use crate::{protocol::{io::{OnionReader, OnionWriter, RawOnionReader, RawOnionWriter}, onion::{Onion, Message, Target, HelloRequest, ClientType}}, crypto::ClientSecret};

pub struct OnionChannel {
    symmetric_cipher: Aes256,
    reader_ref: Mutex<OnionReader<TcpStream, Aes256>>,
    writer_ref: Mutex<OnionWriter<TcpStream, Aes256>>,
    peer_addr: SocketAddr
}

impl OnionChannel {
    pub fn new(stream: TcpStream, symmetric_cipher: Aes256) -> Self {
        Self {
            symmetric_cipher: symmetric_cipher.clone(),
            peer_addr: stream.peer_addr().expect("Failed to retrieve peer address"),
            reader_ref: Mutex::new(RawOnionReader::new(stream.clone()).with_cipher(symmetric_cipher.clone())),
            writer_ref: Mutex::new(RawOnionWriter::new(stream).with_cipher(symmetric_cipher)),
        }
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    pub fn symmetric_cipher(&self) -> Aes256 {
        self.symmetric_cipher.clone()
    }

    pub async fn recv_onion(&self) -> Result<Onion> {
        self.reader_ref.lock().await.read().await
    }

    pub async fn send_onion(&self, onion: Onion) -> Result<()> {
        self.writer_ref.lock().await.write(onion).await
    }

    pub async fn peel_layer(&self, onion: Onion, circuit_id: u32) -> Result<Onion> {
        let payload = if let Message::Payload(payload) = onion.message {
            payload
        } else {
            // err?
            todo!()
        };

        Ok(
            Onion {
                target: Target::Current,
                circuit_id: Some(circuit_id),
                message: Message::Payload(payload)
            }
        )
    }

    pub async fn add_layer(&self, onion: Onion, circuit_id: u32) -> Result<Onion> {
        let mut payload_buf_cursor = Cursor::new(Vec::new());
        RawOnionWriter::new(payload_buf_cursor.get_mut()).with_cipher(self.symmetric_cipher()).write(onion).await?;

        Ok(
            Onion {
                target: Target::Current,
                circuit_id: Some(circuit_id),
                message: Message::Payload(payload_buf_cursor.into_inner())
            }
        )
    }

    pub async fn reach_relay(stream: TcpStream, secret: ClientSecret) -> Result<OnionChannel> {
        let (reader, writer)
            = &mut (RawOnionReader::new(&stream), RawOnionWriter::new(&stream));

        let pub_key = secret.public_key();
    
        writer.write(
            Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::HelloRequest(HelloRequest { client_type: ClientType::Relay, public_key: pub_key })
            }
        ).await?;

        let hello_response = reader.read().await?;

        let symmetric_cipher
            = if let Message::HelloResponse(peer_key) = hello_response.message {
                secret.symmetric_cipher(peer_key).expect("Failed to create symmetric cipher")
            } else {
                //err?
                todo!()
            };

        Ok(OnionChannel::new(stream, symmetric_cipher))
    }
}
