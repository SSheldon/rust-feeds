use std::env;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use fever_api::{ApiRequest, ApiResponse, ApiResponsePayload, Feed};

use models::item::Item as DbItem;

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn query_items() -> Vec<DbItem> {
    use schema::item::dsl::*;

    let connection = establish_connection();
    item.limit(5)
        .load::<DbItem>(&connection)
        .expect("Error loading items")
}

fn fetch_items(feed: &Feed) -> ApiResponsePayload {
    let items: Vec<_> = query_items().into_iter()
        .map(|i| i.into_api_item(feed.id))
        .collect();
    let total_items = items.len() as u32;

    ApiResponsePayload::Items {
        items: items,
        total_items: total_items,
    }
}

pub fn handle_api_request(req_type: &ApiRequest) -> ApiResponse {
    let feed = Feed {
        id: 1,
        title: "xkcd.com".to_owned(),
        url: "https://xkcd.com/".to_owned(),
        is_spark: false,
        last_updated_on_time: NaiveDateTime::from_timestamp(1472799906, 0),
    };

    let payload = match *req_type {
        ApiRequest::Feeds => ApiResponsePayload::Feeds {
            feeds: vec![feed],
            feeds_groups: vec![],
        },
        ApiRequest::Items(_) |
        ApiRequest::ItemsSince(_) |
        ApiRequest::LatestItems => fetch_items(&feed),
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
