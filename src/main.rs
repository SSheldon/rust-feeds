extern crate chrono;
extern crate feed_stream;
extern crate fever_api;
#[macro_use]
extern crate iron;
extern crate serde;
extern crate serde_json;
extern crate url;

use std::collections::HashMap;
use std::io::Read;

use chrono::NaiveDateTime;
use iron::prelude::*;
use iron::method;
use iron::status;
use url::form_urlencoded;

use fever_api::{ApiRequest, ApiResponse, ApiResponsePayload, Feed, Item};

/// Converts a reference to a pair of Strings into a pair of str references.
fn deref_str_pair<'a>(&(ref a, ref b): &'a (String, String))
        -> (&'a str, &'a str) {
    (a, b)
}

fn handle_request(request: &mut Request) -> IronResult<Response> {
    match request.method {
        method::Post => (),
        _ => return Ok(Response::with(status::MethodNotAllowed)),
    }

    let url = request.url.as_ref();
    let query_pairs: Vec<_> = url.query_pairs().into_owned().collect();

    let mut body = Vec::new();
    itry!(request.body.read_to_end(&mut body));
    let body_params: HashMap<_, _> =
        form_urlencoded::parse(&body).into_owned().collect();
    println!("{:?}", body_params);

    let query_pairs = query_pairs.iter().map(deref_str_pair);
    let req_type = iexpect!(ApiRequest::parse(query_pairs, &body_params));
    println!("{:?}", req_type);

    let response = handle_api_request(req_type);
    let response = serde_json::to_string(&response).unwrap();
    println!("{}", response);

    Ok(Response::with((status::Ok, response)))
}

fn handle_api_request(req_type: ApiRequest) -> ApiResponse {
    let feed = Feed {
        id: 1,
        title: "Hello feed".to_owned(),
        url: "example.com".to_owned(),
        is_spark: false,
        last_updated_on_time: NaiveDateTime::from_timestamp(1472799906, 0),
    };

    let item = Item {
        id: 1,
        feed_id: 1,
        title: "Hello item".to_owned(),
        url: "example.com".to_owned(),
        html: "Hello world!".to_owned(),
        is_saved: false,
        is_read: false,
        created_on_time: NaiveDateTime::from_timestamp(1472799806, 0),
    };

    let payload = match req_type {
        ApiRequest::Feeds => ApiResponsePayload::Feeds {
            feeds: vec![feed],
            feeds_groups: vec![],
        },
        ApiRequest::LatestItems |
        ApiRequest::ItemsSince(_) => ApiResponsePayload::Items {
            items: vec![item],
            total_items: 1,
        },
        ApiRequest::UnreadItems => ApiResponsePayload::UnreadItems {
            unread_item_ids: vec![1],
        },
        _ => ApiResponsePayload::None,
    };
    ApiResponse {
        api_version: 1,
        auth: true,
        last_refreshed_on_time: None,
        payload: payload,
    }
}

fn main() {
    Iron::new(handle_request).http("localhost:3000").unwrap();
}
