use std::str::FromStr;

use crate::greader::auth::{AuthHeader, LoginParams, LoginResponse};

pub fn generate_token(username: &str, password: &str) -> String {
    "<token>".to_owned()
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
