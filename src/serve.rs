use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

use futures::future;
use warp::{Filter, self};
use warp::http::StatusCode;

use fever_api::{
    Key as ApiKey,
    Request as ApiRequest,
};

use crate::config::{PgConnectionPool, PooledPgConnection};
use crate::error::Error;
use crate::fetch;
use crate::greader::auth::{
    AuthHeader as GReaderAuthHeader,
    LoginParams as GReaderLoginParams,
    LoginResponse as GReaderLoginResponse,
};
use crate::greader::request::Request as GReaderRequest;
use crate::greader::response::Response as GReaderResponse;
use crate::handling;

impl<T: Debug + Sized + Send + Sync + 'static> warp::reject::Reject for Error<T> { }

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

async fn handle_refresh(
    mut conn: PooledPgConnection,
) -> Result<impl warp::Reply, warp::Rejection> {
    fetch::fetch_items(&mut conn).await
        .map(|_| warp::reply())
        .map_err(|err| warp::reject::custom(err))
}

async fn handle_request(
    request: ApiRequest,
    key: Option<ApiKey>,
    mut conn: PooledPgConnection,
) -> Result<impl warp::Reply, warp::Rejection> {
    let response = handling::handle_api_request(&request, key.as_ref(), &mut conn)
        .map_err(|err| warp::reject::custom(err))?;
    let status = if response.auth {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };
    Ok(warp::reply::with_status(warp::reply::json(&response), status))
}

fn body_string() -> impl Filter<Extract=(String,), Error=warp::Rejection> + Clone {
    async fn read_string(buf: impl bytes::buf::Buf) -> Result<String, warp::Rejection> {
        use std::io::Read;

        let mut result = String::new();
        buf.reader().read_to_string(&mut result)
            .map(|_| result)
            .map_err(fill_err!("Error reading body to string"))
            .map_err(warp::reject::custom)
    }

    warp::body::aggregate()
        .and_then(read_string)
}

async fn handle_greader_login(
    params: GReaderLoginParams,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("{:?}", params);
    let response = GReaderLoginResponse {
        sid: "...".to_owned(),
        lsid: "...".to_owned(),
        auth_token: "<token>".to_owned(),
    };
    Ok(response.to_string())
}

async fn check_greader_auth(
    header: String,
) -> Result<(), warp::Rejection> {
    let auth = GReaderAuthHeader::from_str(&header)
        .map_err(fill_err!("Error parsing authorization header"))
        .map_err(warp::reject::custom)?;
    println!("{:?}", auth.token);
    Ok(())
}

async fn parse_greader_request(
    path: warp::filters::path::Tail,
    params: String,
) -> Result<GReaderRequest, warp::Rejection> {
    GReaderRequest::parse(path.as_str(), &params)
        .ok_or(warp::reject::not_found())
}

impl warp::Reply for GReaderResponse {
    fn into_response(self) -> warp::reply::Response {
        match self {
            GReaderResponse::Plain(s) => s.into_response(),
            GReaderResponse::UserInfo(r) => warp::reply::json(&r).into_response(),
            GReaderResponse::UnreadCount(r) => warp::reply::json(&r).into_response(),
            GReaderResponse::SubscriptionList(r) => warp::reply::json(&r).into_response(),
            GReaderResponse::StreamContents(r) => warp::reply::json(&r).into_response(),
            GReaderResponse::StreamItemsIds(r) => warp::reply::json(&r).into_response(),
            GReaderResponse::StreamItemsContents(r) => warp::reply::json(&r).into_response(),
            GReaderResponse::TagList(r) => warp::reply::json(&r).into_response(),
        }
    }
}

async fn handle_greader_request(
    request: GReaderRequest,
    mut conn: PooledPgConnection,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("{:?}", request);
    let response = crate::greader_handling::handle_api_request(&request, &mut conn);
    println!("{:?}", response);
    response.map_err(|err| warp::reject::custom(err))
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
        .and_then(handle_refresh);

    let fever = warp::path::end()
        .and(api.or(refresh));

    let greader_login = warp::path("accounts")
        .and(warp::path("ClientLogin"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::form())
        .and_then(handle_greader_login);

    let greader_api_base = warp::path("reader")
        .and(warp::path("api"))
        .and(warp::path("0"))
        .and(warp::header("Authorization"))
        .and_then(check_greader_auth)
        .untuple_one()
        .and(warp::path::tail());

    let greader_api_get = greader_api_base
        .and(warp::get())
        .and(warp::query::raw())
        .and_then(parse_greader_request)
        .and(connect_db(pool.clone()))
        .and_then(handle_greader_request);

    let greader_api_post = greader_api_base
        .and(warp::post())
        .and(body_string())
        .and_then(parse_greader_request)
        .and(connect_db(pool.clone()))
        .and_then(handle_greader_request);

    let greader_api = greader_api_get.or(greader_api_post);

    let greader = warp::path("api")
        .and(warp::path("greader.php"))
        .and(greader_login.or(greader_api));

    let route = fever.or(greader).with(warp::log("feeds"));

    warp::serve(route).run(([0, 0, 0, 0], port)).await;
}
