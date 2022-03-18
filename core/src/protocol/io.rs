use super::onion::Onion;
use crate::crypto::SymmetricCipher;
use async_std::io::{Read, Result, Write};

pub struct OnionReader<T: Read, C: SymmetricCipher> {
    reader: T,
    cipher: C,
}

pub struct OnionWriter<T: Write, C: SymmetricCipher> {
    writer: T,
    cipher: C,
}

impl<T: Read, C: SymmetricCipher> OnionReader<T, C> {
    pub fn new(reader: T, cipher: C) -> OnionReader<T, C> {
        OnionReader { reader, cipher }
    }

    pub async fn read(&mut self) -> Result<Onion> {
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
