extern crate chrono;
extern crate md5;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod key;
mod request;
mod response;

pub use key::ApiKey;
pub use request::{ApiRequest, ApiRequestType};
pub use response::{ApiResponse, ApiResponsePayload, Feed, FeedsGroup, Group, Item};

fn join_ids(ids: &[u32], out: &mut String) {
    use std::fmt::Write;

    let mut first = true;
    for &id in ids {
        let sep = if first {""} else {","};
        write!(out, "{}{}", sep, id).unwrap();
        first = false;
    }
}
