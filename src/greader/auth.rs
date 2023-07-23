use std::fmt;
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

use super::request::ParseError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthHeader {
    pub token: String,
}

impl fmt::Display for AuthHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GoogleLogin auth={}", self.token)
    }
}

impl FromStr for AuthHeader {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix("GoogleLogin auth=")
            .map(|s| AuthHeader { token: s.to_owned() })
            .ok_or_else(|| ParseError { type_name: "AuthHeader", value: s.to_owned() })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct LoginParams {
    #[serde(rename = "Email")]
    pub email: String,
    #[serde(rename = "Passwd")]
    pub password: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoginResponse {
    pub sid: String,
    pub lsid: String,
    pub auth_token: String,
}

impl fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SID={}\nLSID={}\nAuth={}", self.sid, self.lsid, self.auth_token)
    }
}
