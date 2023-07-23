use std::str::FromStr;

use base64::prelude::*;
use sha2::{Digest, Sha384};

use crate::greader::auth::{AuthHeader, LoginParams, LoginResponse};

pub fn generate_token(username: &str, password: &str) -> String {
    let mut hasher = Sha384::new();
    hasher.update(username);
    hasher.update(b"\0");
    hasher.update(password);
    let result = hasher.finalize();

    BASE64_URL_SAFE_NO_PAD.encode(result)
}

pub fn handle_login(
    params: &LoginParams,
    creds: Option<(&str, &str)>,
) -> Option<LoginResponse> {
    if let Some((username, password)) = creds {
        if params.email != username || params.password != password {
            return None;
        }
    }

    let response = LoginResponse {
        sid: "...".to_owned(),
        lsid: "...".to_owned(),
        auth_token: generate_token(&params.email, &params.password),
    };
    Some(response)
}

pub fn check(
    header: &str,
    token: Option<&str>,
) -> Result<bool, <AuthHeader as FromStr>::Err> {
    let Some(token) = token else {
        return Ok(true);
    };

    AuthHeader::from_str(&header)
        .map(|auth| auth.token == token)
}
