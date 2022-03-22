use super::{onion::{ Onion, Target }, varint};
use crate::crypto::SymmetricCipher;
use std::{pin::Pin, net::{SocketAddr, Ipv4Addr, IpAddr, Ipv6Addr}};
use async_std::io::{Read, Write, Result, ReadExt, BufReader, ErrorKind, Error};
use super::{bitwriter::BitWriter, varint::VarIntReadable};

pub struct OnionReader<T: Read, C: SymmetricCipher> {
    reader: Pin<Box<BufReader<T>>>,
    cipher: Option<C>,
}

pub struct OnionWriter<T: Write, C: SymmetricCipher> {
    writer: T,
    cipher: C,
}

impl<T: Read, C: SymmetricCipher> OnionReader<T, C> {
    pub fn new(reader: T, cipher: Option<C>) -> Self {
        Self { 
            reader: Box::pin(BufReader::new(reader)), 
            cipher, 
        }
    }

    async fn read_varint<V: VarIntReadable>(&mut self) -> Result<V::Target> {
        let mut buf = [0u8; 6];
        let mut i = 0;
        loop {
            self.reader.read_exact(&mut buf[i..i+1]).await?;
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

    pub async fn read(&mut self) -> Result<Onion> {
        let mut b = [0u8; 1]; 
        self.reader.read_exact(&mut b[0..1]);

        let msgt = b[0].read_bits(5, 3);
        let cip = b[0].read_bits(3, 1);
        let opt1 = b[0].read_bits(2, 1);
        let tgt = b[0].read_bits(0, 2);
        
        let target = match tgt {
            // Relay
            0 => {
               let relay_id = self.read_varint::<u32>().await?;
               Target::Relay(relay_id)
            },
            // IP
            1 => {
                let ipv4 = opt1 == 0;
                let ip = match ipv4 {
                    true => {
                        let mut ip_buf = [0u8; 4];
                        self.reader.read_exact(&mut ip_buf).await?;
                        IpAddr::V4(Ipv4Addr::from(ip_buf))
                    },
                    false => {
                        let mut ip_buf = [0u8; 16];
                        self.reader.read_exact(&mut ip_buf).await?;
                        IpAddr::V6(Ipv6Addr::from(ip_buf))
                    },
                };
                let mut port_buf = [0u8; 2];
                self.reader.read_exact(&mut port_buf);
                let port: u16 = u16::from_be_bytes(port_buf);

                Target::IP(SocketAddr::new(ip, port))
            },
            // Current
            2 => Target::Current,
            _ => panic!("invalid target"),
        };


        let circuit_id = match cip {
            0 => None,
            1 => Some(self.read_varint::<u32>().await?),
            _ => panic!("invalid cip"),
        };

        let message_len: u32 = self.read_varint::<u32>().await?;
        let mut message_raw: Vec<u8> = vec![0u8; message_len as usize];
        self.reader.read_exact(&mut message_raw[..]).await?;

        /*let message = */match msgt {
    /*HelloRequest([u8; 32]),
    HelloResponse(),

    Close(Option<String>),
    Payload(Vec<u8>),

    GetRelaysRequest(),
    GetRelaysResponse(Vec<Relay>),
 
    RelayPingRequest(),
    RelayPingResponse(),*/
            // HelloRequest
            0 => {
                
            },
            // HelloResponse
            1 => {},
            // Close
            2 => {},
            // Payload
            3 => {},
            // GetRelaysRequest
            4 => {},
            // GetRelaysResponse
            

            5 => {},
            _ => panic!("bruh moment"),
        }


    
        //self.reader.read_while()
        /*        let target = match target_type {
            0 => {
                self.reader.read_while(
                self.reader.read_varint
                u32::read_varint(
                Target::Relay()
            }
            1 => { 
                Target::IP()
            }
            2 => Target::Current
            _ => panic!("not implemented");
        }*/

        panic!("not yet implemented");
    }
}

impl<T: Write, C: SymmetricCipher> OnionWriter<T, C> {
    pub fn new(writer: T, cipher: C) -> OnionWriter<T, C> {
        OnionWriter { writer, cipher }
    }

    pub async fn write(&mut self, onion: Onion) -> Result<()> {
        panic!("not yet implemented");
    }
}
