use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use reqwest;

use feed_stream::{Entry, FeedParser};
use fever_api::{
    ApiRequest, ApiRequestType, ApiResponse, ApiResponsePayload,
    Feed as ApiFeed, Item as ApiItem,
};

use data::{ItemsQuery, self};
use models::feed::Feed as DbFeed;
use models::item::{Item as DbItem, NewItem};

fn format_feed(feed: DbFeed) -> ApiFeed {
    ApiFeed {
        id: feed.id as u32,
        title: feed.title,
        url: feed.url,
        is_spark: false,
        last_updated_on_time: NaiveDateTime::from_timestamp(1472799906, 0),
    }
}

fn format_item(item: DbItem) -> ApiItem {
    ApiItem {
        id: item.id as u32,
        feed_id: item.feed_id as u32,
        title: item.title,
        url: item.url,
        html: item.content,
        is_saved: item.is_saved,
        is_read: item.is_read,
        created_on_time: item.published,
    }
}

fn load_feeds(conn: &PgConnection) -> ApiResponsePayload {
    let feeds = data::load_feeds(conn)
        .expect("Error loading feeds")
        .into_iter()
        .map(format_feed)
        .collect();

    ApiResponsePayload::Feeds {
        feeds: feeds,
        feeds_groups: vec![],
    }
}

fn load_items(query: ItemsQuery, conn: &PgConnection) -> ApiResponsePayload {
    let items = data::load_items(query, conn)
        .expect("Error loading items")
        .into_iter()
        .map(format_item)
        .collect();
    let total_items = data::count_items(conn).unwrap();

    ApiResponsePayload::Items {
        items: items,
        total_items: total_items,
    }
}

fn load_unread_item_ids(conn: &PgConnection) -> ApiResponsePayload {
    let ids = data::load_unread_item_ids(conn)
        .expect("Error loading unread item ids")
        .into_iter()
        .map(|i| i as u32)
        .collect();

    ApiResponsePayload::UnreadItems {
        unread_item_ids: ids,
    }
}

fn update_item_read(id: u32, is_read: bool, conn: &PgConnection)
-> ApiResponsePayload {
    use schema::item;

    diesel::update(item::table.find(id as i32))
        .set(item::is_read.eq(is_read))
        .execute(conn)
        .expect("Error updating item is_read");

    load_unread_item_ids(conn)
}

fn item_to_insert_for_entry<'a>(entry: &'a Entry, feed: &DbFeed) -> NewItem<'a> {
    NewItem {
        url: entry.link.as_ref().unwrap(),
        title: &entry.title,
        content: &entry.content,
        published: entry.published.as_ref().map(|d| d.naive_utc()),
        feed_id: feed.id,
    }
}

fn fetch_and_insert_items(feed: &DbFeed, connection: &PgConnection) {
    use schema::item;

    println!("Fetching items from {}...", feed.url);
    let response = if let Ok(response) = reqwest::get(&feed.url) {
        response
    } else {
        println!("Error fetching from {}", feed.url);
        return;
    };

    let parser = FeedParser::new(response);
    let entries: Vec<_> = parser
        .map(|entry| entry.unwrap())
        .take_while(|entry| {
            let link = entry.link.as_ref().unwrap();
            !data::item_already_exists(link, feed, connection).expect("error")
        })
        .collect();

    println!("Found {} new items", entries.len());

    let new_items: Vec<_> = entries.iter()
        .rev()
        .map(|entry| item_to_insert_for_entry(entry, feed))
        .collect();

    diesel::insert_into(item::table)
        .values(&new_items)
        .execute(connection)
        .expect("Error saving new post");
}

pub fn fetch_items_if_needed(conn: &PgConnection) {
    let feeds = data::load_feeds(conn)
        .expect("Error loading feeds");

    for feed in feeds {
        fetch_and_insert_items(&feed, conn);
    }
}

pub fn handle_api_request(request: &ApiRequest, connection: &PgConnection)
-> ApiResponse {
    let payload = match request.req_type {
        ApiRequestType::Feeds => load_feeds(connection),
        ApiRequestType::LatestItems => {
            load_items(ItemsQuery::Latest, connection)
        },
        ApiRequestType::ItemsBefore(id) => {
            load_items(ItemsQuery::Before(id as i32), connection)
        },
        ApiRequestType::ItemsSince(id) => {
            load_items(ItemsQuery::After(id as i32), connection)
        },
        ApiRequestType::Items(ref ids) => {
            let ids: Vec<_> = ids.iter().map(|&i| i as i32).collect();
            load_items(ItemsQuery::ForIds(&ids), connection)
        }
        ApiRequestType::UnreadItems => load_unread_item_ids(connection),
        ApiRequestType::MarkItemRead(id) => update_item_read(id, true, connection),
        ApiRequestType::MarkItemUnread(id) => update_item_read(id, false, connection),
        _ => ApiResponsePayload::None,
    };
    ApiResponse {
        api_version: 1,
        auth: true,
        last_refreshed_on_time: None,
        payload: payload,
    }
}
