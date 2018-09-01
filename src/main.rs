extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate feed_stream;
extern crate fever_api;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate warp;

mod handling;
mod models;
mod schema;

use std::collections::HashMap;
use std::env;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use warp::Filter;

use fever_api::ApiRequest;

/// Converts a reference to a pair of Strings into a pair of str references.
fn deref_str_pair<'a>(&(ref a, ref b): &'a (String, String))
        -> (&'a str, &'a str) {
    (a, b)
}

fn parse_request(
    query_pairs: Vec<(String, String)>,
    body_params: HashMap<String, String>,
) -> Result<ApiRequest, warp::Rejection> {
    let req_type = {
        let query_pairs = query_pairs.iter().map(deref_str_pair);
        ApiRequest::parse(query_pairs, &body_params)
    };

    println!("query: {:?}\nparams: {:?}\nparsed type: {:?}",
        query_pairs, body_params, req_type);

    req_type.ok_or(warp::reject())
}

fn main() {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = Pool::builder()
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .expect("Failed to create pool.");

    handling::fetch_items_if_needed();

    let port = env::var("PORT").ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let api = warp::post2()
        .and(warp::query::<Vec<(String, String)>>())
        .and(warp::body::form::<HashMap<String, String>>())
        .and_then(parse_request);

    let db = warp::any().and_then(move || {
        pool.get().map_err(|_| warp::reject::server_error())
    });

    let route = api
        .and(db)
        .map(|req_type, connection| handling::handle_api_request(&req_type))
        .map(|response| warp::reply::json(&response));

    warp::serve(route).run(([0, 0, 0, 0], port));
}
