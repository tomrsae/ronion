pub(super) trait BitWriter<T> {
    fn write_bits(&mut self, index: u8, bits: T, n: u8);
    fn read_bits(&self, index: u8, n: u8) -> T;
}

impl BitWriter<u8> for u8 {
    fn write_bits(&mut self, index: u8, bits: u8, n: u8) {
        let mask = ((1u32 << n) - 1) as u8;
        let bits_masked = bits & mask;
        let self_masked = *self & !(mask << index);
        *self = self_masked | bits_masked << index;
    }

    fn read_bits(&self, index: u8, n: u8) -> u8 {
        (self >> index) & ((1 << n) - 1)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_bits_full_byte() {
        let mut value = 0u8;
        value.write_bits(0, 0xFF, 8);
        assert_eq!(value, 0xFF);
    }

    #[test]
    fn write_bits_write_ending_bits() {
        let mut value = 0xFFu8;
        value.write_bits(6, 0b01, 2);
        assert_eq!(value, 0b01111111);
    }

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
        let bits = value.read_bits(3, 4);
        assert_eq!(bits, 0b1011);
    }
}
