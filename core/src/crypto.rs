use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

struct KeyPair {
    secret: EphemeralSecret,
    public_key: PublicKey,
}

fn create_key() -> KeyPair {
    let secret = EphemeralSecret::new(OsRng);
    let public_key = PublicKey::from(&secret);
    KeyPair { secret, public_key }
}

fn create_keys(n: u8) -> Vec<KeyPair> {
    let mut keys = Vec::new();
    for _ in 0..n {
        keys.push(create_key())
    }
    keys
}

trait Key {
    fn xor(&mut self, data: &Vec<u8>) -> Vec<u8>;
}

macro_rules! impl_key {
    ($($t:ty),+) => {
        $(impl Key for $t {
            fn xor(&mut self, data: &Vec<u8>) -> Vec<u8> {
                let key_bytes = self.as_bytes();
                let key_len = key_bytes.len();
                let mut res: Vec<u8> = Vec::new();
                for i in 0..data.len() {
                    res.push(data[i] ^ key_bytes[i % key_len])
                }
                return res
            }
        })*
    }
}

impl_key!(SharedSecret, PublicKey);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_key() {}

    #[test]
    fn test_create_keys() {}

    #[test]
    fn test_xor() {
        let mut pair = create_key();

        let msg = "Hello".to_owned().into_bytes();

        let encrypted_msg = pair.public_key.xor(&msg);
        let unencrypted_msg = pair.public_key.xor(&encrypted_msg);

        assert_eq!(msg, unencrypted_msg);
    }
}
