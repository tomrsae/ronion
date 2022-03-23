use super::{onion::{ Onion, Target, Relay }, varint::{self, VarIntWritable}};
use crate::{crypto::SymmetricCipher, protocol::onion::Message};
use core::panic;
use std::{pin::Pin, net::{SocketAddr, Ipv4Addr, IpAddr, Ipv6Addr}, ops::Range};
use async_std::io::{Read, Write, Result, ReadExt, BufReader, ErrorKind, Error, Cursor, BufWriter};
use super::{bitwriter::BitWriter, varint::VarIntReadable};

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
        OnionReader::new(self.reader, cipher)
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
    fn new(reader: Pin<Box<BufReader<R>>>, cipher: C) -> Self {
        Self {reader, cipher}
    }

    pub async fn read(&mut self) -> Result<Onion> {
        read_onion(&mut self.reader).await?;
        let len = read_varint::<BufReader<R>, u32>(&mut self.reader).await?;
        let mut buf: Vec<u8> = vec![0u8; len as usize];
        self.cipher.decrypt(&mut buf);
        self.reader.read_exact(&mut buf[..]);
        read_onion(&mut Box::pin(Cursor::new(buf))).await
    }
}

pub struct RawOnionWriter<T: Write> {
    writer: Pin<Box<BufWriter<T>>>,
}
impl<T: Write> RawOnionWriter<T> {
    pub fn new(writer: T) -> Self {
        let writer = Box::pin(BufWriter::new(writer));
        Self {writer}
    }
    pub fn with_cipher<C: SymmetricCipher>(self, cipher: C) -> OnionWriter<T, C> {
        OnionWriter::new(self.writer, cipher)
    }

    pub async fn write(&mut self, onion: Onion) {
        todo!();
    }
}

pub struct OnionWriter<T: Write, C: SymmetricCipher> {
    writer: Pin<Box<BufWriter<T>>>,
    cipher: C,
}

impl<T: Write, C: SymmetricCipher> OnionWriter<T, C> {
    fn new(writer: Pin<Box<BufWriter<T>>>, cipher: C) -> Self {
        Self {writer, cipher}
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
        match V::from_varint(&buf) {
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

pub fn serialize_relays(relays: &[Relay]) -> Vec<u8> {
    let mut vec = Vec::new();
    for relay in relays {    
        let mut leading = 0u8;
        let ip_bit = if relay.addr.is_ipv6() {1} else {0};
        leading.write_bits(7, ip_bit, 1);
        vec.push(leading);

        match relay.addr.ip() {
            IpAddr::V4(v4) => vec.extend(v4.octets().iter()),
            IpAddr::V6(v6) => vec.extend(v6.octets().iter())
        };

        vec.extend(relay.addr.port().to_be_bytes().iter());

        let (id, id_bytes) = relay.id.to_varint();
        vec.extend(id[0..id_bytes].iter());
    }

    vec
}

pub fn deserialize_relays(data: &[u8]) -> Result<Vec<Relay>> {
    todo!();
    /*    let range_err = Error::new(ErrorKind::InvalidData, "slice out of range");
    let vec = Vec::new();
    
    while data.len() > 0 {
        let ip_bit = data.get(0).ok_or(range_err)?.read_bits(7, 1);
        data = &data[1..];

        let (ip_bytes, ip) = match ip_bit {
            0 => (4, IpAddr::V4(From::<[u8; 4]>::from(data[0..4].try_into().map_err(|_| range_err)?))),
            1 => (16, IpAddr::V6(From::<[u8; 16]>::from(data[0..16].try_into().map_err(|_| range_err)?))), 
            _ => panic!("invalid ip bit")
        };
        data = &data[ip_bytes..];

        let port_bytes: [u8; 2] = data.get(0..2).ok_or(|_| range_err)?.try_into().unwrap(); 
        let port = u16::from_be_bytes(data.get(0..2).map(|x| x.try_into()).ok_or(|_| range_err)?);
        let (id, id_bytes) = u32::from_varint(&data[base..]).map_err(|_| Error::new(ErrorKind::InvalidData, "invalid varint"))?;
        data = &data[2..];

        vec.push(Relay {
            id,
            addr: SocketAddr::new(ip, port),
        });

        data = get(); 
    }

    Ok(vec)*/
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
        _ => return Err(Error::new(ErrorKind::InvalidData, "invalid tgt")),
    };


    let circuit_id = match cip {
        0 => None,
        1 => Some(read_varint::<R, u32>(reader).await?),
        _ => panic!("invalid cip"),
    };

    let message_len: u32 = read_varint::<R, u32>(reader).await?;
    let mut message_raw: Vec<u8> = vec![0u8; message_len as usize];
    reader.read_exact(&mut message_raw[..]).await?;

    let message = match msgt {
        0 => Message::HelloRequest(message_raw.try_into()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid hello request message length"))?),
        1 => Message::HelloResponse(message_raw.try_into()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid hello response message length"))?),
        2 => Message::Close(if message_len > 0 {Some(String::from_utf8_lossy(&message_raw).to_string())} else {None}),
        3 => Message::Payload(message_raw),
        4 => Message::GetRelaysRequest(),
        5 => Message::GetRelaysResponse(deserialize_relays(&message_raw)?),
        6 => Message::RelayPingRequest(),
        7 => Message::RelayPingResponse(),
        _ => panic!("illegal message id"),
    };

    panic!("not yet implemented");
}

