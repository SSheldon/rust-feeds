use std::collections::HashMap;

use futures::Future;
use warp::{Filter, self};

use config::{PgConnectionPool, PooledPgConnection};
use fever_api::ApiRequest;
use handling;

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

pub fn serve(port: u16, pool: PgConnectionPool) {
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
        .and_then(move |_| {
            handling::fetch_items_task(pool.clone())
                .map(|_| warp::reply())
                .map_err(|_| warp::reject::server_error())
        });

    let route = api.or(refresh).with(warp::log("feeds"));

    warp::serve(route).run(([0, 0, 0, 0], port));
}
