use super::onion::{ Onion, Target};
use crate::crypto::SymmetricCipher;
use std::pin::Pin;
use async_std::io::{Read, Result, Write, ReadExt, BufReader, prelude::BufReadExt};
use super::{bitwriter::BitWriter, varint};

pub struct OnionReader<T: Read, C: SymmetricCipher> {
    reader: Pin<Box<BufReader<T>>>,
    cipher: C,
}

pub struct OnionWriter<T: Write, C: SymmetricCipher> {
    writer: T,
    cipher: C,
}

impl<T: Read, C: SymmetricCipher> OnionReader<T, C> {
    pub fn new(reader: T, cipher: C) -> OnionReader<T, C> {
        OnionReader { 
            reader: Box::pin(BufReader::new(reader)), 
            cipher 
        }
    }

    pub async fn read(&mut self) -> Result<Onion> {
        let mut buf = [0u8; 1024];
        self.reader.read_exact(&mut buf[0..1]);
        let target_type = buf[0].read_bits(6, 2);
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
