use std::collections::HashMap;

use futures::Future;
use warp::{Filter, self};
use warp::http::StatusCode;

use config::{PgConnectionPool, PooledPgConnection};
use fever_api::{ApiKey, ApiRequest};
use handling;

fn connect_db(pool: PgConnectionPool)
-> impl Filter<Extract=(PooledPgConnection,), Error=warp::Rejection> + Clone {
    warp::any().and_then(move || {
        pool.get().map_err(|err| warp::reject::custom(err))
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

    request.ok_or_else(warp::reject::not_found)
}

fn is_refresh_request(query_pairs: Vec<(String, String)>) -> bool {
    match query_pairs.first() {
        Some(&(ref action, _)) if action == "refresh" => true,
        _ => false,
    }
}

fn handle_request(
    request: ApiRequest,
    key: Option<&ApiKey>,
    conn: PooledPgConnection,
) -> Result<impl warp::Reply, warp::Rejection> {
    let response = handling::handle_api_request(&request, key, &conn)
        .map_err(|err| warp::reject::custom(err))?;
    let status = if response.auth {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };
    Ok(warp::reply::with_status(warp::reply::json(&response), status))
}

pub fn serve(
    port: u16,
    creds: Option<(String, String)>,
    pool: PgConnectionPool,
) {
    let key = creds.map(|(user, pass)| ApiKey::new(&user, &pass));
    let api = warp::post2()
        .and(warp::query::<Vec<(String, String)>>())
        .and(warp::body::form::<HashMap<String, String>>())
        .and_then(parse_request)
        .and(connect_db(pool.clone()))
        .and_then(move |request, conn| {
            handle_request(request, key.as_ref(), conn)
        });

    let refresh = warp::get2()
        .and(warp::query::<Vec<(String, String)>>())
        .and_then(|query| {
            if is_refresh_request(query) { Ok(()) }
            else { Err(warp::reject::not_found()) }
        })
        .and_then(move |_| {
            handling::fetch_items_task(pool.clone())
                .map(|_| warp::reply())
                .map_err(|_| warp::reject::not_found())
        });

    let route = api.or(refresh).with(warp::log("feeds"));

    warp::serve(route).run(([0, 0, 0, 0], port));
}
