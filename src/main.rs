extern crate chrono;
extern crate feed_stream;
extern crate fever_api;
#[macro_use]
extern crate iron;
extern crate reqwest;
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

use feed_stream::{Entry, FeedParser};
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

fn item_from_entry(entry: Entry, id: u32, feed: &Feed) -> Item {
    Item {
        id: id,
        feed_id: feed.id,
        title: entry.title().to_owned(),
        url: entry.link().unwrap().to_owned(),
        html: entry.content().unwrap().into_owned(),
        is_saved: false,
        is_read: false,
        created_on_time: entry.published().unwrap().naive_utc(),
    }
}

fn fetch_items(feed: &Feed) -> ApiResponsePayload {
    let response = reqwest::get("https://xkcd.com/atom.xml").unwrap();
    let parser = FeedParser::new(response);
    let items: Vec<_> = parser
        .map(|entry| entry.unwrap())
        .enumerate()
        .map(|(i, entry)| item_from_entry(entry, 100 - (i as u32), feed))
        .collect();
    let total_items = items.len() as u32;

    ApiResponsePayload::Items {
        items: items,
        total_items: total_items,
    }
}

fn handle_api_request(req_type: ApiRequest) -> ApiResponse {
    let feed = Feed {
        id: 1,
        title: "xkcd.com".to_owned(),
        url: "https://xkcd.com/".to_owned(),
        is_spark: false,
        last_updated_on_time: NaiveDateTime::from_timestamp(1472799906, 0),
    };

    let payload = match req_type {
        ApiRequest::Feeds => ApiResponsePayload::Feeds {
            feeds: vec![feed],
            feeds_groups: vec![],
        },
        ApiRequest::LatestItems |
        ApiRequest::ItemsSince(_) => fetch_items(&feed),
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
