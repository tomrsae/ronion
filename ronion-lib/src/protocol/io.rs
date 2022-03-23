use super::{onion::{ Onion, Target }, varint};
use crate::crypto::SymmetricCipher;
use std::{pin::Pin, net::{SocketAddr, Ipv4Addr, IpAddr, Ipv6Addr}, ops::DerefMut};
use async_std::io::{Read, Write, Result, ReadExt, BufReader, ErrorKind, Error, Cursor};
use super::{bitwriter::BitWriter, varint::VarIntReadable};

enum MessageType {
    HelloRequest = 0,
    HelloResponse = 1,

    Close = 2,
    Payload = 3,

    GetRelaysRequest = 4,
    GetRelaysResponse = 5,

    RelayPingRequest = 6,
    RelayPingResponse = 7,
}
impl MessageType {
    fn to_u8(self) -> u8 {
        self as u8
    }
}


pub struct RawOnionReader<T: Read> { 
    reader: Pin<Box<BufReader<T>>>,
}

impl<T: Read> RawOnionReader<T> {
    pub fn new(reader: T) -> Self {
        Self { 
            reader: Box::pin(BufReader::new(reader)) 
        }          
    }

    pub fn with_cipher<C: SymmetricCipher>(self, cipher: C) -> OnionReader<T, C> {
        OnionReader { 
            reader: self.reader,
            cipher,
        }
    }

    pub async fn read(&mut self) -> Result<Onion> {
        read_onion(&mut self.reader).await
    }
}


pub struct OnionReader<R: Read, C: SymmetricCipher> {
    reader: Pin<Box<BufReader<R>>>,
    cipher: C
}

impl<R: Read, C: SymmetricCipher> OnionReader<R, C> {
    async fn rvarint(r: &mut Pin<Box<R>>) {
        read_varint::<R, u32>(r);
    }

    pub async fn read(&mut self) -> Result<Onion> {
        read_onion(&mut self.reader).await?;
        let len = read_varint::<BufReader<R>, u32>(&mut self.reader).await?;
        let mut buf: Vec<u8> = vec![0u8; len as usize];
        self.reader.read_exact(&mut buf[..]);
        read_onion(&mut Box::pin(Cursor::new(buf))).await
    }
}


pub struct OnionWriter<T: Write, C: SymmetricCipher> {
    writer: T,
    cipher: C,
}

impl<T: Write, C: SymmetricCipher> OnionWriter<T, C> {
    pub fn new(writer: T, cipher: C) -> OnionWriter<T, C> {
        OnionWriter { writer, cipher }
    }

    pub async fn write(&mut self, onion: Onion) -> Result<()> {
        panic!("not yet implemented");
    }
}


async fn read_varint<R: Read, V: VarIntReadable>(reader: &mut Pin<Box<R>>) -> Result<V::Target> {
    let mut buf = [0u8; 6];
    let mut i = 0;
    loop {
        reader.read_exact(&mut buf[i..i+1]).await?;
        match V::read_varint(&buf) {
            Ok((value, _bytes)) => {
                return Ok(value);
            },
            Err(varint::Error::Malformed) => {
               // not enough data, continue 
            },
            Err(varint::Error::Overflow) => {
                return Err(Error::new(ErrorKind::InvalidData, "varint overflow"));
            },
        }
        i += 1;
    }
}

pub async fn read_onion<R: Read>(reader: &mut Pin<Box<R>>) -> Result<Onion> {
    let mut b = [0u8; 1]; 
    reader.read_exact(&mut b[0..1]);

    let msgt = b[0].read_bits(5, 3);
    let cip = b[0].read_bits(3, 1);
    let opt1 = b[0].read_bits(2, 1);
    let tgt = b[0].read_bits(0, 2);
    
    let target = match tgt {
        // Relay
        0 => {
           let relay_id = read_varint::<R, u32>(reader).await?;
           Target::Relay(relay_id)
        },
        // IP
        1 => {
            let ipv4 = opt1 == 0;
            let ip = match ipv4 {
                true => {
                    let mut ip_buf = [0u8; 4];
                    reader.read_exact(&mut ip_buf).await?;
                    IpAddr::V4(Ipv4Addr::from(ip_buf))
                },
                false => {
                    let mut ip_buf = [0u8; 16];
                    reader.read_exact(&mut ip_buf).await?;
                    IpAddr::V6(Ipv6Addr::from(ip_buf))
                },
            };
            let mut port_buf = [0u8; 2];
            reader.read_exact(&mut port_buf);
            let port: u16 = u16::from_be_bytes(port_buf);

            Target::IP(SocketAddr::new(ip, port))
        },
        // Current
        2 => Target::Current,
        _ => panic!("invalid target"),
    };


    let circuit_id = match cip {
        0 => None,
        1 => Some(read_varint::<R, u32>(reader).await?),
        _ => panic!("invalid cip"),
    };

    let message_len: u32 = read_varint::<R, u32>(reader).await?;
    let mut message_raw: Vec<u8> = vec![0u8; message_len as usize];
    reader.read_exact(&mut message_raw[..]).await?;

    /*let message = match msgt {
        MessageType::HelloRequest.to_u8() => {
            if message_len != 32 {
                return Err(Error::new(ErrorKind::InvalidData, "hello had a non-32 byte key"));
            }
            Message::HelloRequest(message_raw[0..32])
        },
        /* HelloResponse
        1 => {},
        // Close
        2 => {},
        // Payload
        3 => {},
        // GetRelaysRequest
        4 => {},
        // GetRelaysResponse
        

        5 => {},*/
        _ => panic!("bruh moment"),
    };*/

    panic!("not yet implemented");
}

