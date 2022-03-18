pub(crate) trait VarIntRW<T> {
    fn read_varint(b: &[u8]) -> T;
    fn write_varint(&self, b: &mut [u8]);
}

macro_rules! unsigned_varint {
    ($t:ty) => {
       impl VarIntRW for $t {
           fn read_varint(b: &[u8]) -> T {
               panic!("nimpl");
           }

           fn write_varint(&self, b: &mut [u8]) {
               panic!("nimpl");
           }
       }
    };
}
