use super::bitwriter::BitWriter;

pub(super) trait VarIntReadable<T> {
    fn read_varint(b: &[u8]) -> T;
}
pub(super) trait VarIntWritable<T> {
    fn write_varint(&self, b: &mut [u8]);
}

macro_rules! unsigned_impl {
    ($t:ty) => {
       impl VarIntReadable<$t> for $t {
           fn read_varint(b: &[u8]) -> $t {
                let mut value: $t = 0;
                let mut more = 1u8;
                let mut i = 0;
                let mut shift = 0;
                while more != 0 {
                    let bits = b[i].read_bits(0, 7) as $t;
                    value = value | bits << shift;
                    more = b[i].read_bits(7, 1);
                    shift += 7;
                    i += 1;
                }
                <$t>::from_le(value)
            }
        }

        impl VarIntWritable<$t> for $t {
           fn write_varint(&self, b: &mut [u8]) {
                let mut value = self.to_le();
                let mut i = 0;
                while value != 0 {
                    let bits = ((value & 0b01111111) | 0b10000000) as u8;
                    b[i] = bits;
                    b[i].write_bits(0, bits, 8);
                    value >>= 7;
                    i += 1;
                }
                // reset the 'more' bit on the final byte
                b[i - 1].write_bits(7, 0, 1);
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

        value.write_varint(&mut buf); 

        assert_eq!(buf, [0b10110011, 0b11010100, 0b00000010]);
    }

    #[test]
    fn read_varint_u32() {
        let buf = [0b10110011, 0b11010100, 0b00000010];
        
        let value = u32::read_varint(&buf);
        
        assert_eq!(value, 0b10_1010100_0110011u32);
    }

    #[test]
    fn read_varint_can_read_output_of_write() {
        let mut buf = [0u8; 5];
        let expected = 0xDEADBEEF;
        expected.write_varint(&mut buf);
        let actual = u32::read_varint(&buf);
        
        assert_eq!(actual, expected);
    }
}
