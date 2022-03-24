use super::{onion::{ Onion, Target, Relay }, varint::{self, VarIntWritable}};
use crate::{crypto::SymmetricCipher, protocol::onion::Message};
use core::panic;
use std::{pin::Pin, net::{SocketAddr, Ipv4Addr, IpAddr, Ipv6Addr}, ops::Range, borrow::Borrow};
use async_std::io::{Read, Write, Result, ReadExt, BufReader, ErrorKind, Error, Cursor, BufWriter, WriteExt};
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

    pub async fn write(&mut self, onion: Onion) -> Result<()> {
        write_onion(&mut self.writer, onion).await
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
    let mut buf = [0u8; u32::MAX_VARINT_LEN];
    let mut i = 0;
    loop {
        reader.read_exact(&mut buf[i..i+1]).await?;
        i += 1;
        match V::from_varint(&buf[..i]) {
            Ok((value, _bytes)) => {
                println!("RETURNING!");
                return Ok(value);
            },
            Err(varint::Error::Malformed) => {
                println!("MALFORMED!");
               // not enough data, continue 
            },
            Err(varint::Error::Overflow) => {
                return Err(Error::new(ErrorKind::InvalidData, "varint overflow"));
            },
        }
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
        vec.extend(relay.pub_key.iter());

        let (id, id_bytes) = relay.id.to_varint();
        vec.extend(id[0..id_bytes].iter());
    }
    
    vec
}

pub fn deserialize_relays(mut data: &[u8]) -> Result<Vec<Relay>> {
    let range_err = || Error::new(ErrorKind::InvalidData, "slice out of range");
    let mut vec = Vec::new();
    
    while data.len() > 0 {
        let ip_bit = data.get(0).ok_or_else(range_err)?.read_bits(7, 1);
        data = &data[1..];
        println!("IP");
        let (ip_bytes, ip) = match ip_bit {
            0 => (4, IpAddr::V4(From::<[u8; 4]>::from(data.get(0..4).ok_or_else(range_err)?.try_into().unwrap()))),
            1 => (16, IpAddr::V6(From::<[u8; 16]>::from(data.get(0..16).ok_or_else(range_err)?.try_into().unwrap()))),  
            _ => panic!("invalid ip bit")
        };
        data = &data[ip_bytes..];

        let port = u16::from_be_bytes(data.get(0..2).ok_or_else(range_err)?.try_into().unwrap()); 
        data = &data[2..];

        let pub_key = data.get(0..32).ok_or_else(range_err)?.try_into().unwrap();
        data = &data[32..];

        let (id, id_bytes) = u32::from_varint(data).map_err(|_| Error::new(ErrorKind::InvalidData, "invalid varint"))?;
        data = &data[id_bytes..];


        vec.push(Relay {
            id,
            pub_key,
            addr: SocketAddr::new(ip, port),
        });
    }

    Ok(vec)
}

pub async fn read_onion<R: Read>(reader: &mut Pin<Box<R>>) -> Result<Onion> {
    let mut b = [0u8; 1]; 
    reader.read_exact(&mut b[0..1]).await?;

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
            reader.read_exact(&mut port_buf).await?;
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
    println!("TARGET: {:?}, CIRCUIT_ID: {:?}, MSGLEN: {}", target, circuit_id, message_len);

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

    Ok(Onion {circuit_id, message, target})
}

pub async fn write_onion<'a, W: Write>(writer: &mut Pin<Box<BufWriter<W>>>, onion: Onion) -> Result<()> {
    let mut buf = [0u8; 128];
    let target_index = 1;

    let (tgt, opt1, offset) = match onion.target {
        Target::Relay(id) => {
            let len = id.write_varint(&mut buf[1..]).unwrap();
            (0, 0, len)
        },
        Target::IP(addr) => {
            let (ip_len, opt1) = match addr.ip() {
                IpAddr::V4(v4) => {
                    let octets = v4.octets();
                    buf[target_index..].iter_mut().zip(octets).for_each(|(dst, src)| *dst = src);
                    (octets.len(), 0)
                }
                IpAddr::V6(v6) => {
                    let octets = v6.octets();
                    buf[target_index..].iter_mut().zip(octets).for_each(|(dst, src)| *dst = src);
                    (octets.len(), 1)
                }
            };
            let port_index = target_index + ip_len;
            let port_bytes = addr.port().to_be_bytes();
            buf[port_index..].iter_mut().zip(port_bytes).for_each(|(dst, src)| *dst = src);
            (1, opt1, ip_len + port_bytes.len()) 
        },
        Target::Current => (2, 0, 0usize),
    };

    let circuit_id_index = target_index +  offset;
    let (cip, offset) = match onion.circuit_id {
        Some(id) => (1, id.write_varint(&mut buf[circuit_id_index..]).unwrap()),
        None => (0, 0),
    };
    println!("WR CIRCUIT_ID: {:?}", &buf[circuit_id_index..circuit_id_index+offset]);
    let message_len_index = circuit_id_index + offset;

    // TODO: refactor so this variable isnt needed
    let mut message_vec = None;
    let (msgt, message_len) = match onion.message {
        Message::HelloRequest(ref data) => (0, data.len()),
        Message::HelloResponse(ref data) => (1, data.len()),
        Message::Close(ref text) => (2, text.as_ref().map_or(0, |x| x.as_bytes().len())), 
        Message::Payload(ref data) => (3, data.len()),
        Message::GetRelaysRequest() => (4, 0),
        Message::GetRelaysResponse(ref data) => {
            let vec = serialize_relays(&data[..]);
            let len = vec.len();
            message_vec = Some(vec);
            (5, len)
        },
        Message::RelayPingRequest() => (6, 0),
        Message::RelayPingResponse() => (7, 0)
    };

    buf[0].write_bits(5, msgt, 3);
    buf[0].write_bits(3, cip, 1);
    buf[0].write_bits(2, opt1, 1);
    buf[0].write_bits(0, tgt, 2);

    let offset = message_len.write_varint(&mut buf[message_len_index..]).unwrap();
    let message_index = message_len_index + offset;
    writer.write(&buf[..message_index]).await?;

    match onion.message {
        Message::HelloRequest(public_key) => writer.write(&public_key[..]).await?,
        Message::HelloResponse(signed_public_key) => writer.write(&signed_public_key[..]).await?,
        Message::Close(text) => writer.write(text.as_ref().map_or(&[] as &[u8], |x| x.as_bytes())).await?,
        Message::Payload(data) => writer.write(&data[..]).await?,
        Message::GetRelaysRequest() => 0,
        Message::GetRelaysResponse(_relays) => writer.write(&message_vec.unwrap()).await?,
        Message::RelayPingRequest() => 0,
        Message::RelayPingResponse() => 0
    };

    writer.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! onion_rw_test {
        ($name:ident, $onion:expr) => {
            #[async_std::test]
            async fn $name() {
                let mut cursor = Cursor::new(Vec::new());
                let mut raw_writer = RawOnionWriter::new(cursor.get_mut());

                raw_writer.write($onion).await.unwrap();

                cursor.set_position(0);
                let mut raw_reader = RawOnionReader::new(cursor);
                let onion = raw_reader.read().await.unwrap();

                assert_eq!($onion, onion); 
            }
        }
    }

    macro_rules! onion_rw_message_test {
        ($name:ident, $message:expr) => {
            onion_rw_test!($name, Onion {
                circuit_id: None,
                target: Target::IP(SocketAddr::new(IpAddr::from(Ipv4Addr::new(1,2,3,4)), 1337)),
                message: $message,
            });
        }
    }
     
    onion_rw_test!(onion_read_write_ipv4_empty_payload_with_circuit_id, Onion{
        circuit_id: Some(0xBEEF),
        message: Message::Payload(Vec::new()),
        target: Target::IP(SocketAddr::new(IpAddr::from(Ipv4Addr::new(1,2,3,4)), 1337)),
    });

    onion_rw_test!(onion_read_write_ipv4_empty_payload_without_circuit_id, Onion{
        circuit_id: None,
        message: Message::Payload(Vec::new()),
        target: Target::IP(SocketAddr::new(IpAddr::from(Ipv4Addr::new(1,2,3,4)), 1337)),
    });

    onion_rw_test!(onion_read_write_ipv6_empty_payload_with_circuit_id, Onion{
        circuit_id: Some(1),
        message: Message::Payload(Vec::new()),
        target: Target::IP(SocketAddr::new(IpAddr::from(Ipv6Addr::new(0xDEAD, 0xBEEF, 0xCAFE, 0xBABE, 0x1CE, 0xF00, 0xC173, 0xFEED)), 1337)),
    });


    onion_rw_message_test!(
        onion_read_write_message_hello_request,
        Message::HelloRequest([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1])
    );
    
    onion_rw_message_test!(
        onion_read_write_message_hello_response,
        Message::HelloResponse([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
        ])
    );

    onion_rw_message_test!(
        onion_read_write_message_close_empty,
        Message::Close(None)
    );

    onion_rw_message_test!(onion_read_write_message_get_relays_request, Message::GetRelaysRequest());

    onion_rw_message_test!(onion_read_write_message_get_relays_response, Message::GetRelaysResponse(vec![Relay {
        id: 0xBEEF,
        addr: SocketAddr::new(IpAddr::from(Ipv4Addr::new(100, 120, 140, 160)), 0xBEEF),
        pub_key: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1],
    }]));


    onion_rw_message_test!(onion_read_write_message_relay_ping_request, Message::RelayPingRequest());
    onion_rw_message_test!(onion_read_write_message_relay_ping_response, Message::RelayPingResponse());
}
