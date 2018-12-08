mod key;
mod request;
mod response;

pub use crate::key::Key;
pub use crate::request::{Request, RequestType};
pub use crate::response::{Response, ResponsePayload, Feed, FeedsGroup, Group, Item};

fn join_ids(ids: &[u32], out: &mut String) {
    use std::fmt::Write;

    let mut first = true;
    for &id in ids {
        let sep = if first {""} else {","};
        write!(out, "{}{}", sep, id).unwrap();
        first = false;
    }
}
