use std::fmt;
use std::str::FromStr;

use md5::Md5;
use md5::digest::{Digest, FixedOutput};

#[derive(Clone, PartialEq)]
pub struct Key([u8; 16]);

impl Key {
    pub fn new(username: &str, password: &str) -> Key {
        let mut hash = Md5::new();
        hash.update(username.as_bytes());
        hash.update(":".as_bytes());
        hash.update(password.as_bytes());
        Key(hash.finalize_fixed().into())
    }
}

fn val_from_hex_char(c: u8) -> Result<u8, ()> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(()),
    }
}

fn byte_from_hex(s: &[u8]) -> Result<u8, ()> {
    Ok(val_from_hex_char(s[0])? * 16 + val_from_hex_char(s[1])?)
}

impl FromStr for Key {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        if s.len() != 32 {
            return Err(());
        }

        let mut bytes = [0; 16];
        for (byte, hex) in bytes.iter_mut().zip(s.as_bytes().chunks(2)) {
            *byte = byte_from_hex(hex)?;
        }

        Ok(Key(bytes))
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0.iter() {
            fmt::LowerHex::fmt(byte, f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, byte_from_hex};

    // Expected signature for "user:password"
    static SAMPLE_BYTES: [u8; 16] = [
        0x3a, 0xe9, 0xea, 0x5f, 0xe7, 0xad, 0x5b, 0xf6,
        0x52, 0xc5, 0x1f, 0x43, 0xda, 0x57, 0x42, 0x2c,
    ];
    static SAMPLE_HEX: &'static str = "3ae9ea5fe7ad5bf652c51f43da57422c";

    #[test]
    fn test_formatting() {
        let key = Key(SAMPLE_BYTES);
        assert_eq!(key.to_string(), SAMPLE_HEX);
    }

    #[test]
    fn test_parsing() {
        let byte = byte_from_hex(b"3a");
        assert_eq!(byte, Ok(0x3a));

        let key = SAMPLE_HEX.parse::<Key>();
        assert_eq!(key, Ok(Key(SAMPLE_BYTES)));
    }

    #[test]
    fn test_signing() {
        let key = Key::new("user", "password");
        assert_eq!(key, Key(SAMPLE_BYTES));
    }
}
