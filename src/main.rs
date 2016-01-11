#[macro_use]
extern crate iron;

use std::io::Read;

use iron::prelude::*;
use iron::method;
use iron::status;

#[derive(Debug, PartialEq)]
enum ApiRequest {
    None,
    Groups,
    Feeds,
    LatestItems,
    ItemsSince(u32),
    ItemsBefore(u32),
    Items(Vec<u32>),
    UnreadItems,
    SavedItems,
    MarkItemRead(u32),
    MarkItemSaved(u32),
    MarkItemUnsaved(u32),
    MarkFeedRead(u32, u32),
    MarkGroupRead(u32, u32),
}

fn request_type(query: &str) -> Option<ApiRequest> {
    let components: Vec<_> = query.split('&').map(|c| {
        let mut split = c.splitn(2, '=');
        (split.next().unwrap(), split.next())
    }).collect();

    let components = match components.split_first() {
        Some((&("api", None), components)) => components,
        _ => return None,
    };
    let (action, components) = match components.split_first() {
        Some((&action, components)) => (action, components),
        None => return Some(ApiRequest::None),
    };

    match action {
        ("groups", _) => Some(ApiRequest::Groups),
        ("feeds", _) => Some(ApiRequest::Feeds),
        ("items", _) => {
            unimplemented!();
        }
        ("unread_item_ids", _) => Some(ApiRequest::UnreadItems),
        ("saved_item_ids", _) => Some(ApiRequest::SavedItems),
        ("mark", Some("item")) => {
            unimplemented!();
        }
        ("mark", Some("feed")) => {
            unimplemented!();
        }
        ("mark", Some("group")) => {
            unimplemented!();
        }
        _ => None,
    }
}

fn handle_request(request: &mut Request) -> IronResult<Response> {
    match request.method {
        method::Post => (),
        _ => return Ok(Response::with(status::MethodNotAllowed)),
    }

    let query = iexpect!(request.url.query.as_ref());
    let req_type = iexpect!(request_type(query));
    println!("{:?}", req_type);

    let mut body = String::new();
    itry!(request.body.read_to_string(&mut body));
    println!("{}", body);

    Ok(Response::with((status::Ok, "{\"api_version\":1,\"auth\":1}")))
}

fn main() {
    Iron::new(handle_request).http("localhost:3000").unwrap();
}

#[cfg(test)]
mod tests {
    use super::{ApiRequest, request_type};

    #[test]
    fn test_request_type() {
        assert_eq!(request_type("api"), Some(ApiRequest::None));
        assert_eq!(request_type("api&feeds"), Some(ApiRequest::Feeds));
        assert_eq!(request_type("api&unread_item_ids"), Some(ApiRequest::UnreadItems));
    }
}
