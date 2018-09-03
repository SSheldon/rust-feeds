use std::borrow::Cow;

use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use reqwest;

use feed_stream::{Entry, FeedParser};
use fever_api::{ApiRequest, ApiResponse, ApiResponsePayload, Feed};

use models::item::{Item as DbItem, NewItem};

fn load_feeds() -> ApiResponsePayload {
    let feed = Feed {
        id: 1,
        title: "xkcd.com".to_owned(),
        url: "https://xkcd.com/".to_owned(),
        is_spark: false,
        last_updated_on_time: NaiveDateTime::from_timestamp(1472799906, 0),
    };

    ApiResponsePayload::Feeds {
        feeds: vec![feed],
        feeds_groups: vec![],
    }
}

fn query_items(connection: &PgConnection) -> Vec<DbItem> {
    DbItem::query(connection, None, None).expect("Error loading items")
}

fn load_items(connection: &PgConnection) -> ApiResponsePayload {
    let items: Vec<_> = query_items(connection)
        .into_iter()
        .map(DbItem::into_api_item)
        .collect();
    let total_items = items.len() as u32;

    ApiResponsePayload::Items {
        items: items,
        total_items: total_items,
    }
}

fn item_to_insert_for_entry(entry: &Entry) -> NewItem {
    let content = match entry.content() {
        Some(Cow::Borrowed(content)) => content,
        _ => panic!("too lazy to handle this case"),
    };

    NewItem {
        url: entry.link().unwrap(),
        title: entry.title(),
        content: content,
        published: entry.published().map(|d| d.naive_utc()),
    }
}

fn fetch_and_insert_items(connection: &PgConnection) {
    use schema::item;

    let response = reqwest::get("https://xkcd.com/atom.xml").unwrap();
    let parser = FeedParser::new(response);
    let entries: Vec<_> = parser
        .map(|entry| entry.unwrap())
        .collect();

    let new_items: Vec<_> = entries.iter()
        .map(item_to_insert_for_entry)
        .collect();

    diesel::insert_into(item::table)
        .values(&new_items)
        .execute(connection)
        .expect("Error saving new post");
}

pub fn fetch_items_if_needed(connection: &PgConnection) {
    if query_items(connection).is_empty() {
        fetch_and_insert_items(connection);
    }
}

pub fn handle_api_request(req_type: &ApiRequest, connection: &PgConnection)
-> ApiResponse {
    let payload = match *req_type {
        ApiRequest::Feeds => load_feeds(),
        ApiRequest::Items(_) |
        ApiRequest::ItemsSince(_) |
        ApiRequest::LatestItems => load_items(connection),
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
