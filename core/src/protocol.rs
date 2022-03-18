use async_std::{net::IpAddr, io::{Result, Read, Write}};

use crate::crypto::SymmetricCipher;

type RelayID = u32;

pub enum Target {
    Relay(RelayID),
    IP(IpAddr),
    Current,
}

pub struct Onion {
    target: Target,
    payload: Vec<u8>,
}

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
        OnionReader { 
            reader, 
            cipher,
        }
    }

    pub async fn read() -> Result<Onion> {
        panic!("not yet implemented");
    }
}

impl<T: Write, C: SymmetricCipher> OnionWriter<T, C> {
    pub fn new(writer: T, cipher: C) -> OnionWriter<T, C> {
        OnionWriter {
            writer,
            cipher,
        }
    }

    pub async fn write(onion: Onion) -> Result<()> {
        panic!("not yet implemented");
    }
}

trait BitWriter<T> {
    fn write_bits(&mut self, index: u8, bits: T, n: u8);
    fn read_bits(&self, index: u8, n: u8) -> T;
}

impl BitWriter<u8> for u8 {
    fn write_bits(&mut self, index: u8, bits: u8, n: u8) {
        let bits_masked = bits & ((1 << n) - 1);
        *self |= bits_masked << index;
    }

    fn read_bits(&self, index: u8, n: u8) -> u8 {
        (self >> (index - n)) & ((1 << n) - 1)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_bits_ands_bits_according_to_index() {
        let mut value = 0u8;
        value.write_bits(3, 0b1010, 4);
        assert_eq!(value, 0b1010000);
    }

    #[test]
    fn write_bits_masks_bits_that_are_written() {
        let mut value = 0u8;
        value.write_bits(4, 0b1011, 2);
        assert_eq!(value, 0b110000);
    }

    #[test]
    fn read_bits_reads_n_bits_at_specified_index() {
        let value = 0b11011000;
        let bits = value.read_bits(7, 4);
        assert_eq!(bits, 0b1011);
    }
}
