extern crate iron;

use std::io::Read;

use iron::prelude::*;
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

fn request_type(query: &str) -> ApiRequest {
    let components: Vec<_> = query.split('&').map(|c| {
        let mut split = c.splitn(2, '=');
        (split.next().unwrap(), split.next())
    }).collect();

    let components = match components.split_first() {
        Some((&("api", _), components)) => components,
        _ => return ApiRequest::None,
    };
    let (action, components) = match components.split_first() {
        Some((&action, components)) => (action, components),
        None => return ApiRequest::None,
    };

    match action {
        ("groups", _) => ApiRequest::Groups,
        ("feeds", _) => ApiRequest::Feeds,
        ("items", _) => {
            unimplemented!();
        }
        ("unread_item_ids", _) => ApiRequest::UnreadItems,
        ("saved_item_ids", _) => ApiRequest::SavedItems,
        ("mark", Some("item")) => {
            unimplemented!();
        }
        ("mark", Some("feed")) => {
            unimplemented!();
        }
        ("mark", Some("group")) => {
            unimplemented!();
        }
        _ => ApiRequest::None,
    }
}

fn handle_request(request: &mut Request) -> IronResult<Response> {
    let query = request.url.query.as_ref().unwrap();
    let req_type = request_type(query);
    println!("{:?}", req_type);

    let mut body = String::new();
    request.body.read_to_string(&mut body).unwrap();
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
        assert_eq!(request_type("api"), ApiRequest::None);
        assert_eq!(request_type("api&feeds"), ApiRequest::Feeds);
        assert_eq!(request_type("api&unread_item_ids"), ApiRequest::UnreadItems);
    }
}
