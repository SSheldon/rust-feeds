use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::query_dsl::LoadQuery;
use reqwest;

use feed_stream::{Entry, FeedParser};
use fever_api::{
    ApiRequest, ApiRequestType, ApiResponse, ApiResponsePayload,
    Item as ApiItem,
};

use models::feed::{Feed as DbFeed, NewFeed};
use models::item::{Item as DbItem, NewItem};

fn format_item(item: DbItem) -> ApiItem {
    ApiItem {
        id: item.id as u32,
        feed_id: item.feed_id as u32,
        title: item.title,
        url: item.url,
        html: item.content,
        is_saved: false,
        is_read: false,
        created_on_time: item.published,
    }
}

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
        .map(format_item)
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

fn item_already_exists(entry: &Entry, feed: &DbFeed, conn: &PgConnection) -> bool {
    use diesel::dsl::{exists, select};
    use schema::item::dsl::*;

    let link = entry.link.as_ref().unwrap();
    let query = item.filter(feed_id.eq(feed.id).and(url.eq(link)));
    select(exists(query))
        .get_result(conn)
        .expect("error")
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
        .take_while(|entry| !item_already_exists(entry, feed, connection))
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
    let feeds = DbFeed::load(conn)
        .expect("Error loading feeds");

    let feeds = if feeds.is_empty() {
        vec![insert_feed(conn)]
    } else {
        feeds
    };

    for feed in feeds {
        fetch_and_insert_items(&feed, conn);
    }
}

pub fn handle_api_request(request: &ApiRequest, connection: &PgConnection)
-> ApiResponse {
    let payload = match request.req_type {
        ApiRequestType::Feeds => load_feeds(connection),
        ApiRequestType::LatestItems => {
            load_items(DbItem::latest_query(), connection)
        },
        ApiRequestType::ItemsBefore(id) => {
            load_items(DbItem::before_query(id as i32), connection)
        },
        ApiRequestType::ItemsSince(id) => {
            load_items(DbItem::after_query(id as i32), connection)
        },
        ApiRequestType::Items(ref ids) => {
            let ids: Vec<_> = ids.iter().map(|&i| i as i32).collect();
            load_items(DbItem::for_ids_query(&ids), connection)
        }
        ApiRequestType::UnreadItems => load_unread_item_ids(connection),
        _ => ApiResponsePayload::None,
    };
    ApiResponse {
        api_version: 1,
        auth: true,
        last_refreshed_on_time: None,
        payload: payload,
    }
}
