extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate feed_stream;
extern crate fever_api;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate warp;

mod data;
mod handling;
mod models;
mod schema;

use std::collections::HashMap;
use std::env;

use diesel::pg::PgConnection;
use warp::Filter;

use fever_api::ApiRequest;

type PgConnectionManager = diesel::r2d2::ConnectionManager<PgConnection>;
type PgConnectionPool = diesel::r2d2::Pool<PgConnectionManager>;
type PooledPgConnection = diesel::r2d2::PooledConnection<PgConnectionManager>;

fn connect_db(pool: PgConnectionPool)
-> impl Filter<Extract=(PooledPgConnection,), Error=warp::Rejection> + Clone {
    warp::any().and_then(move || {
        pool.get().map_err(|_| warp::reject::server_error())
    })
}

/// Converts a reference to a pair of Strings into a pair of str references.
fn deref_str_pair<'a>(&(ref a, ref b): &'a (String, String))
-> (&'a str, &'a str) {
    (a, b)
}

fn parse_request(
    query_pairs: Vec<(String, String)>,
    body_params: HashMap<String, String>,
) -> Result<ApiRequest, warp::Rejection> {
    let request = {
        let query_pairs = query_pairs.iter().map(deref_str_pair);
        ApiRequest::parse(query_pairs, &body_params)
    };

    println!("query: {:?}\nparams: {:?}\nparsed: {:?}",
        query_pairs, body_params, request);

    request.ok_or_else(warp::reject)
}

fn is_refresh_request(query_pairs: Vec<(String, String)>) -> bool {
    match query_pairs.first() {
        Some(&(ref action, _)) if action == "refresh" => true,
        _ => false,
    }
}

fn handle_request(
    request: ApiRequest,
    conn: PooledPgConnection,
) -> impl warp::Reply {
    let response = handling::handle_api_request(&request, &conn);
    warp::reply::json(&response)
}

fn handle_refresh(conn: PooledPgConnection) -> impl warp::Reply {
    handling::fetch_items_if_needed(&conn);
    warp::reply()
}

fn main() {
    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgConnectionPool::new(PgConnectionManager::new(database_url))
        .expect("Failed to create pool.");

    let connection = pool.get().expect("Failed to retrieve connection.");
    handling::fetch_items_if_needed(&connection);

    let port = env::var("PORT").ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let api = warp::post2()
        .and(warp::query::<Vec<(String, String)>>())
        .and(warp::body::form::<HashMap<String, String>>())
        .and_then(parse_request)
        .and(connect_db(pool.clone()))
        .map(handle_request);

    let refresh = warp::get2()
        .and(warp::query::<Vec<(String, String)>>())
        .and_then(|query| {
            if is_refresh_request(query) { Ok(()) }
            else { Err(warp::reject()) }
        })
        .and(connect_db(pool.clone()))
        .map(|_, conn| handle_refresh(conn));

    let route = api.or(refresh).with(warp::log("feeds"));

    warp::serve(route).run(([0, 0, 0, 0], port));
}
