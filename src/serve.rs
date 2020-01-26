use std::collections::HashMap;

use futures::{TryFutureExt, future};
use warp::{Filter, self};
use warp::http::StatusCode;

use fever_api::{
    Key as ApiKey,
    Request as ApiRequest,
};

use crate::config::{MaybePooled, PgConnectionPool, PooledPgConnection};
use crate::error::Error;
use crate::fetch;
use crate::handling;

impl warp::reject::Reject for Error<diesel::result::Error> { }
impl warp::reject::Reject for Error<diesel::r2d2::PoolError> { }

fn connect_db(pool: PgConnectionPool)
-> impl Filter<Extract=(PooledPgConnection,), Error=warp::Rejection> + Clone {
    warp::any().and_then(move || {
        let conn = pool.get()
            .map_err(fill_err!("Error getting connection from pool"))
            .map_err(|err| warp::reject::custom(err));
        future::ready(conn)
    })
}

/// Converts a reference to a pair of Strings into a pair of str references.
fn deref_str_pair<'a>(&(ref a, ref b): &'a (String, String))
-> (&'a str, &'a str) {
    (a, b)
}

async fn parse_request(
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

async fn accept_refresh(
    query_pairs: Vec<(String, String)>,
) -> Result<(), warp::Rejection> {
    let is_refresh = match query_pairs.first() {
        Some(&(ref action, _)) if action == "refresh" => true,
        _ => false,
    };

    if is_refresh {
        Ok(())
    } else {
        Err(warp::reject::not_found())
    }
}

async fn handle_request(
    request: ApiRequest,
    key: Option<ApiKey>,
    conn: PooledPgConnection,
) -> Result<impl warp::Reply, warp::Rejection> {
    let response = handling::handle_api_request(&request, key.as_ref(), &conn)
        .map_err(|err| warp::reject::custom(err))?;
    let status = if response.auth {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };
    Ok(warp::reply::with_status(warp::reply::json(&response), status))
}

pub async fn serve(
    port: u16,
    creds: Option<(String, String)>,
    pool: PgConnectionPool,
) {
    let key = creds.map(|(user, pass)| ApiKey::new(&user, &pass));
    let api = warp::post()
        .and(warp::query::<Vec<(String, String)>>())
        .and(warp::body::form::<HashMap<String, String>>())
        .and_then(parse_request)
        .and(connect_db(pool.clone()))
        .and_then(move |request, conn| {
            handle_request(request, key.clone(), conn)
        });

    let refresh = warp::get()
        .and(warp::query::<Vec<(String, String)>>())
        .and_then(accept_refresh)
        .untuple_one()
        .and(connect_db(pool.clone()))
        .and_then(move |conn| {
            fetch::fetch_items(MaybePooled::Pooled(conn))
                .map_ok(|_| warp::reply())
                .map_err(|err| warp::reject::custom(err))
        });

    let route = api.or(refresh).with(warp::log("feeds"));

    warp::serve(route).run(([0, 0, 0, 0], port)).await;
}
