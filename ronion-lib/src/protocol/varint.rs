use super::bitwriter::BitWriter;
use std::result::Result;

#[derive(Debug)]
#[derive(PartialEq)]
pub(super) enum Error {
    Overflow,
    Malformed,
}

pub(super) trait VarIntReadable<T> {
    /// Reads a VarInt from a buffer.
    /// Returns either an error or a tuple of (value, bytes_read).
    fn read_varint(b: &[u8]) -> Result<(T, usize), Error>;
}
pub(super) trait VarIntWritable<T> {
    /// Writes a VarInt into a buffer.
    /// Returns either an error or the amount of bytes written.
    fn write_varint(&self, b: &mut [u8]) -> Result<usize, Error>;
}

macro_rules! unsigned_impl {
    ($t:ty) => {
       impl VarIntReadable<$t> for $t {
           fn read_varint(b: &[u8]) -> Result<($t, usize), Error> {
                let mut value: $t = 0;
                let mut more = 1u8;
                let mut i = 0;
                let mut shift = 0;
                while more != 0 {
                    if shift >= <$t>::BITS {
                        return Err(Error::Overflow);
                    }
                    if i >= b.len() {
                        return Err(Error::Malformed);
                    }
                    let bits = b[i].read_bits(0, 7) as $t;
                    value = value | bits << shift;
                    more = b[i].read_bits(7, 1);
                    shift += 7;
                    i += 1;
                }
                Ok((<$t>::from_le(value), i))
            }
        }

        impl VarIntWritable<$t> for $t {
           fn write_varint(&self, b: &mut [u8]) -> Result<usize, Error> {
                let mut value = self.to_le();
                let mut i = 0;
                while value != 0 {
                    if i >= b.len() {
                        return Err(Error::Overflow);
                    }
                    let bits = ((value & 0b01111111) | 0b10000000) as u8;
                    b[i] = bits;
                    b[i].write_bits(0, bits, 8);
                    value >>= 7;
                    i += 1;
                }
                // reset the 'more' bit on the final byte
                b[i - 1].write_bits(7, 0, 1);
                Ok(i)
           }
       }
    };
}
macro_rules! unsigned_impls {
    ( $($t:ty), * ) => {
        $(
            unsigned_impl!($t);
        )*
    };
}

unsigned_impls!(u8, u16, u32, u64, u128, usize);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_varint_u32() {
        let value = 0b10_1010100_0110011u32;
        let mut buf = [0u8; 3];

        let bytes = value.write_varint(&mut buf).unwrap(); 

        assert_eq!(bytes, 3);
        assert_eq!(buf, [0b10110011, 0b11010100, 0b00000010]);
    }

    #[test]
    fn read_varint_u32() {
        let buf = [0b10110011, 0b11010100, 0b00000010];
        
        let (value, bytes) = u32::read_varint(&buf).unwrap();
        
        assert_eq!(value, 0b10_1010100_0110011u32);
        assert_eq!(bytes, 3);
    }

    #[test]
    fn read_varint_u32_overflow() {
        let more = 1u8 << 7;
        let buf = [more; 16];

        let value = u32::read_varint(&buf).unwrap_err();
        
        assert_eq!(value, Error::Overflow);
    }

    #[test]
    fn read_varint_u32_malformed() {
        let buf = [0b10000000];
        
        let err = u32::read_varint(&buf).unwrap_err();
        
        assert_eq!(err, Error::Malformed);
    }

    #[test]
    fn write_varint_u32_insufficient_buffer() {
        let mut buf = [0b10000000u8; 1];

        let err = 0xDEADBEEFu32.write_varint(&mut buf).unwrap_err();

        assert_eq!(err, Error::Overflow);
    }

    #[test]
    fn read_varint_can_read_output_of_write() {
        let mut buf = [0u8; 5];
        let expected = 0xDEADBEEF;
        expected.write_varint(&mut buf).unwrap();
        let (actual, _) = u32::read_varint(&buf).unwrap();
        
        assert_eq!(actual, expected);
    }
}
