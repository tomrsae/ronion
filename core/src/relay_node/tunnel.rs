use std::{net::SocketAddr, cell::RefCell, sync::Arc};

use crate::crypto::Aes256;
use async_std::{net::TcpStream, io::{Result, Read, Write, Cursor}, sync::Mutex};

use crate::{protocol::{io::{OnionReader, OnionWriter, RawOnionReader, RawOnionWriter}, onion::{Onion, Message, Target, HelloRequest, ClientType}}, crypto::ClientSecret};

pub struct Tunnel {
    symmetric_cipher: Aes256,
    reader: Mutex<OnionReader<TcpStream, Aes256>>,
    writer: Mutex<OnionWriter<TcpStream, Aes256>>,
    peer_addr: SocketAddr
}

impl Tunnel {
    pub fn new(stream: TcpStream, symmetric_cipher: Aes256) -> Self {
        Self {
            symmetric_cipher: symmetric_cipher.clone(),
            peer_addr: stream.peer_addr().expect("Failed to retrieve peer address"),
            reader: Mutex::new(RawOnionReader::new(stream.clone()).with_cipher(symmetric_cipher.clone())),
            writer: Mutex::new(RawOnionWriter::new(stream).with_cipher(symmetric_cipher)),
        }
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    pub async fn recv_onion(&self) -> Onion {
        self.reader.lock().await.read().await.expect("Failed to read onion")
    }

    pub async fn send_onion(&self, onion: Onion) -> Result<()> {
        self.writer.lock().await.write(onion).await
    }

    pub async fn peel_layer(&self, payload: Vec<u8>, symmetric_cipher: Aes256) -> Result<Onion> {
        let cursor = Cursor::new(payload);

        Ok(RawOnionReader::new(cursor).with_cipher(symmetric_cipher).read().await?)
    }

    pub async fn add_layer(&self, onion: Onion) -> Result<Vec<u8>> {
        let mut payload_buf_cursor = Cursor::new(Vec::new());
        RawOnionWriter::new(payload_buf_cursor.get_mut()).with_cipher(self.symmetric_cipher.clone()).write(onion).await?;

        Ok(payload_buf_cursor.into_inner())
    }

    pub async fn reach_relay(stream: TcpStream, secret: ClientSecret) -> Result<Tunnel> {
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

        let symmetric_cipher = if let Message::HelloResponse(peer_key) = hello_response.message {
            secret.symmetric_cipher(peer_key).expect("Failed to create symmetric cipher")
        } else {
            //err?
            todo!()
        };

        Ok(Tunnel::new(stream, symmetric_cipher))
    }
}
