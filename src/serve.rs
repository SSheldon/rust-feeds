use std::collections::HashMap;
use std::fmt::Debug;

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
use crate::greader::request::RequestType as GReaderRequestType;
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

async fn parse_greader_request(
    path: warp::filters::path::Tail,
    params: String,
) -> Result<GReaderRequestType, warp::Rejection> {
    GReaderRequestType::parse(path.as_str(), &params)
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
    request: GReaderRequestType,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("{:?}", request);
    let response: GReaderResponse = match request {
        GReaderRequestType::UserInfo => crate::greader::response::UserInfoResponse {
            user_id: "123".to_owned(),
            user_name: "Name".to_owned(),
            user_profile_id: "123".to_owned(),
            user_email: "username@gmail.com".to_owned(),
            is_blogger_user: true,
            signup_time: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            public_user_name: "username".to_owned(),
        }.into(),
        GReaderRequestType::TagList => crate::greader::response::TagListResponse {
            tags: vec![],
        }.into(),
        GReaderRequestType::SubscriptionList => crate::greader::response::SubscriptionListResponse {
            subscriptions: vec![
                crate::greader::response::Subscription {
                    title: "xkcd".to_owned(),
                    first_item_time: chrono::NaiveDateTime::from_timestamp_opt(1373999174, 0).unwrap(),
                    html_url: "https://xkcd.com/".to_owned(),
                    sort_id: "A1".to_owned(),
                    id: crate::greader::request::StreamId::Feed("1".to_owned()),
                    categories: vec![],
                },
            ],
        }.into(),
        GReaderRequestType::StreamItemsIds(_) => crate::greader::response::StreamItemsIdsResponse {
            item_refs: vec![
                crate::greader::response::ItemRef {
                    id: crate::greader::request::ItemId(1),
                    timestamp: chrono::NaiveDateTime::from_timestamp_opt(1405538280, 0).unwrap(),
                    direct_stream_ids: vec![],
                },
            ],
        }.into(),
        GReaderRequestType::StreamItemsContents(_) => crate::greader::response::StreamItemsContentsResponse {
            items: vec![
                crate::greader::response::Item {
                    updated: chrono::NaiveDateTime::from_timestamp_opt(1405538866, 0).unwrap(),
                    id: crate::greader::request::ItemId(1),
                    categories: vec![],
                    author: "Author".to_owned(),
                    alternate: vec![
                        crate::greader::response::Link {
                            href: "http://example.com".to_owned(),
                            link_type: Some("text/html".to_owned()),
                        },
                    ],
                    timestamp: chrono::NaiveDateTime::from_timestamp_opt(1405538280, 0).unwrap(),
                    content: crate::greader::response::ItemContent {
                        direction: "ltr".to_owned(),
                        content: "Hello world!".to_owned(),
                    },
                    crawl_time: chrono::NaiveDateTime::from_timestamp_opt(1405538280, 0).unwrap(),
                    published: chrono::NaiveDateTime::from_timestamp_opt(1405538280, 0).unwrap(),
                    title: "Title".to_owned(),
                },
            ],
        }.into(),
        _ => "OK".to_owned().into(),
    };
    println!("{:?}", response);
    Ok(response)
}

fn greader_api() -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
    let api = warp::path("reader")
        .and(warp::path("api"))
        .and(warp::path("0"))
        .and(warp::path::tail());

    let api_get = api
        .and(warp::get())
        .and(warp::query::raw())
        .and_then(parse_greader_request)
        .and_then(handle_greader_request);

    let api_post = api
        .and(warp::post())
        .and(body_string())
        .and_then(parse_greader_request)
        .and_then(handle_greader_request);

    api_get.or(api_post)
}

fn greader_auth() -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
    warp::path("accounts")
        .and(warp::path("ClientLogin"))
        .map(|| "SID=...\nLSID=...\nAuth=<token>")
}

fn greader() -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("greader.php"))
        .and(greader_auth().or(greader_api()))
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

    let route = greader().or(api).or(refresh).with(warp::log("feeds"));

    warp::serve(route).run(([0, 0, 0, 0], port)).await;
}
