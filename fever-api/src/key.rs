use std::fmt::{Display, self};
use std::str::FromStr;

use md5::{Digest, Md5};

pub struct ApiKey([u8; 16]);

impl ApiKey {
    pub fn new(username: &str, password: &str) -> ApiKey {
        let mut hash = Md5::new();
        hash.input(username.as_bytes());
        hash.input(":".as_bytes());
        hash.input(password.as_bytes());
        ApiKey(hash.hash())
    }
}

impl FromStr for ApiKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Err(())
    }
}

impl Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0.iter() {
            fmt::LowerHex::fmt(byte, f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ApiKey;

    #[test]
    fn test_signing() {
        let key = ApiKey::new("user", "password");
        assert_eq!(key.to_string(), "3ae9ea5fe7ad5bf652c51f43da57422c");
    }
}
