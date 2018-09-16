use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::query_dsl::LoadQuery;
use reqwest;

use feed_stream::{Entry, FeedParser};
use fever_api::{ApiRequest, ApiResponse, ApiResponsePayload};

use models::feed::{Feed as DbFeed, NewFeed};
use models::item::{Item as DbItem, NewItem};

fn load_feeds(conn: &PgConnection) -> ApiResponsePayload {
    let feeds = DbFeed::load(conn)
        .expect("Error loading feeds")
        .into_iter()
        .map(DbFeed::into_api_feed)
        .collect();

    ApiResponsePayload::Feeds {
        feeds: feeds,
        feeds_groups: vec![],
    }
}

fn load_items<Q>(query: Q, conn: &PgConnection) -> ApiResponsePayload
where Q: RunQueryDsl<PgConnection> + LoadQuery<PgConnection, DbItem> {
    let items = query.load::<DbItem>(conn)
        .expect("Error loading items")
        .into_iter()
        .map(DbItem::into_api_item)
        .collect();
    let total_items = DbItem::count(conn).unwrap();

    ApiResponsePayload::Items {
        items: items,
        total_items: total_items,
    }
}

fn load_unread_item_ids(conn: &PgConnection) -> ApiResponsePayload {
    let ids = {
        use schema::item::dsl::*;

        // TODO: filter out read items
        item.select(id)
            .load::<i32>(conn)
            .expect("Error loading unread item ids")
    };

    let ids = ids.into_iter()
        .map(|i| i as u32)
        .collect();

    ApiResponsePayload::UnreadItems {
        unread_item_ids: ids,
    }
}

fn insert_feed(conn: &PgConnection) -> DbFeed {
    use schema::feed;

    let new_feed = NewFeed {
        url: "https://xkcd.com/atom.xml",
        title: "xkcd",
    };

    diesel::insert_into(feed::table)
        .values(&new_feed)
        .get_result(conn)
        .expect("Error saving new feed")
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

    let response = reqwest::get(&feed.url).unwrap();
    let parser = FeedParser::new(response);
    let entries: Vec<_> = parser
        .map(|entry| entry.unwrap())
        .collect();

    let new_items: Vec<_> = entries.iter()
        .map(|entry| item_to_insert_for_entry(entry, feed))
        .collect();

    diesel::insert_into(item::table)
        .values(&new_items)
        .execute(connection)
        .expect("Error saving new post");
}

pub fn fetch_items_if_needed(connection: &PgConnection) {
    let count = DbItem::count(connection)
        .expect("Error counting items");
    if count == 0 {
        let feed = insert_feed(connection);
        fetch_and_insert_items(&feed, connection);
    }
}

pub fn handle_api_request(req_type: &ApiRequest, connection: &PgConnection)
-> ApiResponse {
    let payload = match *req_type {
        ApiRequest::Feeds => load_feeds(connection),
        ApiRequest::LatestItems => {
            load_items(DbItem::latest_query(), connection)
        },
        ApiRequest::ItemsBefore(id) => {
            load_items(DbItem::before_query(id as i32), connection)
        },
        ApiRequest::ItemsSince(id) => {
            load_items(DbItem::after_query(id as i32), connection)
        },
        ApiRequest::UnreadItems => load_unread_item_ids(connection),
        _ => ApiResponsePayload::None,
    };
    ApiResponse {
        api_version: 1,
        auth: true,
        last_refreshed_on_time: None,
        payload: payload,
    }
}
