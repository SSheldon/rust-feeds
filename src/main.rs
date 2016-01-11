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

impl ApiRequest {
    pub fn parse(query: &str) -> Option<ApiRequest> {
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
            ("groups", None) => Some(ApiRequest::Groups),
            ("feeds", None) => Some(ApiRequest::Feeds),
            ("favicons", None) => unimplemented!(),
            ("items", None) => match components.first() {
                Some(&("since_id", val)) => val.and_then(|v| v.parse().ok())
                                               .map(|v| ApiRequest::ItemsSince(v)),
                Some(&("max_id", val)) => val.and_then(|v| v.parse().ok())
                                             .map(|v| ApiRequest::ItemsBefore(v)),
                Some(&("with_ids", val)) => val.and_then(|v| {
                    let ids: Result<_, _> = v.split(',').map(|v| v.parse()).collect();
                    ids.map(|v| ApiRequest::Items(v)).ok()
                }),
                None => Some(ApiRequest::LatestItems),
                _ => None,
            },
            ("links", None) => unimplemented!(),
            ("unread_item_ids", None) => Some(ApiRequest::UnreadItems),
            ("saved_item_ids", None) => Some(ApiRequest::SavedItems),
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

    pub fn query(&self) -> String {
        use std::fmt::Write;
        use ApiRequest::*;

        match *self {
            None => "api".to_owned(),
            Groups => "api&groups".to_owned(),
            Feeds => "api&feeds".to_owned(),
            LatestItems => "api&items".to_owned(),
            ItemsSince(id) =>
                format!("api&items&since_id={}", id),
            ItemsBefore(id) =>
                format!("api&items&max_id={}", id),
            Items(ref ids) => {
                let mut result = "api&items&with_ids=".to_owned();
                let mut first = true;
                for &id in ids {
                    let sep = if first {""} else {","};
                    write!(&mut result, "{}{}", sep, id).unwrap();
                    first = false;
                }
                result
            }
            UnreadItems => "api&unread_item_ids".to_owned(),
            SavedItems => "api&saved_item_ids".to_owned(),
            MarkItemRead(id) =>
                format!("api&mark=item&as=read&id={}", id),
            MarkItemSaved(id) =>
                format!("api&mark=item&as=saved&id={}", id),
            MarkItemUnsaved(id) =>
                format!("api&mark=item&as=unsaved&id={}", id),
            MarkFeedRead(id, before) =>
                format!("api&mark=feed&as=read&id={}&before={}", id, before),
            MarkGroupRead(id, before) =>
                format!("api&mark=group&as=read&id={}&before={}", id, before),
        }
    }
}

fn handle_request(request: &mut Request) -> IronResult<Response> {
    match request.method {
        method::Post => (),
        _ => return Ok(Response::with(status::MethodNotAllowed)),
    }

    let query = iexpect!(request.url.query.as_ref());
    let req_type = iexpect!(ApiRequest::parse(query));
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
    use super::ApiRequest;

    #[test]
    fn test_request_type() {
        assert_eq!(ApiRequest::parse("api"), Some(ApiRequest::None));
        assert_eq!(ApiRequest::parse("api&feeds"), Some(ApiRequest::Feeds));
        assert_eq!(ApiRequest::parse("api&unread_item_ids"),
                   Some(ApiRequest::UnreadItems));
        assert_eq!(ApiRequest::parse("api&items&since_id=0"),
                   Some(ApiRequest::ItemsSince(0)));
        assert_eq!(ApiRequest::parse("api&items&with_ids=0,1,2"),
                   Some(ApiRequest::Items(vec![0, 1, 2])));
    }
}
