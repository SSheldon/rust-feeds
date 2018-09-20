use std::fmt::{Display, self};
use std::str::FromStr;

pub struct ApiKey([u8; 16]);

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
