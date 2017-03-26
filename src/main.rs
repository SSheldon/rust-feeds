extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate feed_stream;
extern crate fever_api;
#[macro_use]
extern crate iron;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate url;

mod handling;
mod models;
mod schema;

use std::collections::HashMap;
use std::env;
use std::io::Read;

use iron::prelude::*;
use iron::method;
use iron::status;
use url::form_urlencoded;

use fever_api::ApiRequest;

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

    let query_pairs = query_pairs.iter().map(deref_str_pair);
    let req_type = iexpect!(ApiRequest::parse(query_pairs, &body_params));

    let response = handling::handle_api_request(&req_type);
    let response = itry!(serde_json::to_string(&response));

    println!("url: {}\nparams: {:?}\nparsed type: {:?}\nresponse: {}",
        url, body_params, req_type, response);

    Ok(Response::with((status::Ok, response)))
}

fn main() {
    let port = env::var("PORT").ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    Iron::new(handle_request).http(("0.0.0.0", port)).unwrap();
}
