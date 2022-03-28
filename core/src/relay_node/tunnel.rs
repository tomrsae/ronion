use std::{cell::RefCell, net::SocketAddr, sync::Arc};

use crate::crypto::Aes256;
use async_std::{
    io::{Cursor, Read, Result, Write},
    net::TcpStream,
    sync::Mutex,
};

use crate::{
    crypto::ClientSecret,
    protocol::{
        io::{OnionReader, OnionWriter, RawOnionReader, RawOnionWriter},
        onion::{ClientType, HelloRequest, Message, Onion, Target},
    },
};

pub struct OnionTunnel {
    symmetric_cipher: Aes256,
    reader: Mutex<OnionReader<TcpStream, Aes256>>,
    writer: Mutex<OnionWriter<TcpStream, Aes256>>,
    peer_addr: SocketAddr,
}

impl OnionTunnel {
    // Returns a new OnionTunnel on which to read and write onions
    // param stream: The connection to establish the tunnel on
    // param symmetric_cipher: The symmetric cipher between the sender and receiver used in securing the tunnel
    pub fn new(stream: TcpStream, symmetric_cipher: Aes256) -> Self {
        Self {
            symmetric_cipher: symmetric_cipher.clone(),
            peer_addr: stream.peer_addr().expect("Failed to retrieve peer address"),
            reader: Mutex::new(
                RawOnionReader::new(stream.clone()).with_cipher(symmetric_cipher.clone()),
            ),
            writer: Mutex::new(RawOnionWriter::new(stream).with_cipher(symmetric_cipher)),
        }
    }

    // Returns the socket address of the other side of the tunnel
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    // Reads the connection for onions and returns the read onion
    pub async fn recv_onion(&self) -> Onion {
        self.reader
            .lock()
            .await
            .read()
            .await
            .expect("Failed to read onion")
    }

    // Writes an onion on the connection
    pub async fn send_onion(&self, onion: Onion) -> Result<()> {
        self.writer.lock().await.write(onion).await
    }

    // Peels a layer of encryption from the given payload, returning a peeled onion
    // param payload: The payload buffer to peel
    // param symmetric_cipher: The symmetric cipher used to peel away the layer of encryption
    pub async fn peel_layer(&self, payload: Vec<u8>, symmetric_cipher: Aes256) -> Result<Onion> {
        let cursor = Cursor::new(payload);

        Ok(RawOnionReader::new(cursor)
            .with_cipher(symmetric_cipher)
            .read()
            .await?)
    }

    // Adds a layer of encryption on an onion, returning a byte buffer containing the encrypted onion
    // param onion: The onion to add a layer of encryption on
    pub async fn add_layer(&self, onion: Onion) -> Result<Vec<u8>> {
        let mut payload_buf_cursor = Cursor::new(Vec::new());
        RawOnionWriter::new(payload_buf_cursor.get_mut())
            .with_cipher(self.symmetric_cipher.clone())
            .write(onion)
            .await?;

        Ok(payload_buf_cursor.into_inner())
    }

    // A static implementation used to directly create a secure onion tunnel between two relays
    // param stream: The connection to create the onion tunnel on
    // param secret: The secret to use in the establishment of the tunnel
    pub async fn reach_relay(stream: TcpStream, secret: ClientSecret) -> Result<OnionTunnel> {
        let (reader, writer) = &mut (RawOnionReader::new(&stream), RawOnionWriter::new(&stream));

        let pub_key = secret.public_key();

        writer
            .write(Onion {
                target: Target::Current,
                circuit_id: None,
                message: Message::HelloRequest(HelloRequest {
                    client_type: ClientType::Relay,
                    public_key: pub_key,
                }),
            })
            .await?;

        let hello_response = reader.read().await?;

        let symmetric_cipher = if let Message::HelloResponse(peer_key) = hello_response.message {
            secret
                .symmetric_cipher(peer_key)
                .expect("Failed to create symmetric cipher")
        } else {
            //err?
            todo!()
        };

        Ok(OnionTunnel::new(stream, symmetric_cipher))
    }
}
