use std::collections::HashMap;
use std::ops::Deref;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use futures::{Future, Stream, stream};
use iter_read::IterRead;
use reqwest;
use reqwest::async::{Chunk, Client};

use feed_stream::{Entry, FeedParser};
use fever_api::{
    ApiRequest, ApiRequestType, ApiResponse, ApiResponsePayload, self,
};

use data::{ItemsQuery, self};
use models::feed::Feed as DbFeed;
use models::group::Group as DbGroup;
use models::item::{Item as DbItem, NewItem};

fn format_group(group: DbGroup) -> fever_api::Group {
    fever_api::Group {
        id: group.id as u32,
        title: group.title,
    }
}

fn format_feed(feed: DbFeed) -> fever_api::Feed {
    fever_api::Feed {
        id: feed.id as u32,
        title: feed.title,
        url: feed.url,
        site_url: None,
        is_spark: false,
        last_updated_on_time: None,
    }
}

fn format_feeds_groups(feed_groups: impl Iterator<Item=(i32, Option<i32>)>)
-> Vec<fever_api::FeedsGroup> {
    let mut groups: HashMap<u32, Vec<u32>> = HashMap::new();
    for (feed_id, group_id) in feed_groups {
        if let Some(group_id) = group_id {
            groups.entry(group_id as u32).or_default().push(feed_id as u32)
        }
    }

    groups.into_iter()
        .map(|(group_id, feed_ids)| fever_api::FeedsGroup { group_id, feed_ids })
        .collect()
}

fn format_item(item: DbItem) -> fever_api::Item {
    fever_api::Item {
        id: item.id as u32,
        feed_id: item.feed_id as u32,
        title: item.title,
        author: None,
        url: item.url,
        html: item.content,
        is_saved: item.is_saved,
        is_read: item.is_read,
        created_on_time: item.published,
    }
}

fn load_groups(conn: &PgConnection) -> ApiResponsePayload {
    let groups = data::load_groups(conn)
        .expect("Error loading groups")
        .into_iter()
        .map(format_group)
        .collect();

    let feed_groups = data::load_feed_groups(conn)
        .expect("Error loading feeds");
    let feeds_groups = format_feeds_groups(feed_groups.into_iter());

    ApiResponsePayload::Groups { groups, feeds_groups }
}

fn load_feeds(conn: &PgConnection) -> ApiResponsePayload {
    let feeds = data::load_feeds(conn)
        .expect("Error loading feeds");

    let feeds_groups = {
        let feed_groups = feeds.iter()
            .map(|feed| (feed.id, feed.group_id));

        format_feeds_groups(feed_groups)
    };

    let feeds = feeds
        .into_iter()
        .map(format_feed)
        .collect();

    ApiResponsePayload::Feeds {
        feeds: feeds,
        feeds_groups: feeds_groups,
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

fn load_saved_item_ids(conn: &PgConnection) -> ApiResponsePayload {
    let ids = data::load_saved_item_ids(conn)
        .expect("Error loading saved item ids")
        .into_iter()
        .map(|i| i as u32)
        .collect();

    ApiResponsePayload::SavedItems {
        saved_item_ids: ids,
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

fn update_item_saved(id: u32, is_saved: bool, conn: &PgConnection)
-> ApiResponsePayload {
    use schema::item;

    diesel::update(item::table.find(id as i32))
        .set(item::is_saved.eq(is_saved))
        .execute(conn)
        .expect("Error updating item is_saved");

    load_saved_item_ids(conn)
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

fn parse_new_entries(
    response: &[Chunk],
    feed: &DbFeed,
    connection: &PgConnection,
) -> Vec<Entry> {
    let chunks = response.iter().map(|chunk| -> &[u8] { &chunk });
    let parser = FeedParser::new(IterRead::new(chunks));
    parser
        .map(|entry| entry.unwrap())
        .take_while(|entry| {
            let link = entry.link.as_ref().unwrap();
            !data::item_already_exists(link, feed, connection).expect("error")
        })
        .collect()
}

fn fetch_feed(feed: &DbFeed, client: &Client)
-> impl Future<Item=Vec<Chunk>, Error=reqwest::Error> + 'static {
    println!("Fetching items from {}...", feed.url);
    client.get(&feed.url).send().and_then(|response| {
        response.into_body().collect()
    })
}

fn fetch_feeds(feeds: &[DbFeed])
-> impl Stream<Item=Vec<Chunk>, Error=reqwest::Error> + 'static {
    let client = Client::new();
    let feed_responses: Vec<_> = feeds.iter()
        .map(move |feed| fetch_feed(feed, &client))
        .collect();

    stream::futures_ordered(feed_responses)
}

fn insert_new_feed_items<'a>(
    iter: impl Iterator<Item=(&'a DbFeed, Result<Vec<Chunk>, reqwest::Error>)>,
    conn: &'a PgConnection,
) {
    use schema::item;

    let feed_entries: Vec<_> = iter
        .filter_map(|(feed, response)| {
            match response {
                Ok(response) => {
                    let entries = parse_new_entries(&response, feed, conn);
                    println!("Found {} new items for {}", entries.len(), feed.url);
                    Some((feed, entries))
                },
                Err(err) => {
                    println!("Error fetching from {}: {}", feed.url, err);
                    None
                }
            }
        })
        .collect();

    let mut new_items = Vec::new();
    for &(feed, ref entries) in feed_entries.iter() {
        for entry in entries.iter().rev() {
            new_items.push(item_to_insert_for_entry(entry, feed));
        }
    }

    diesel::insert_into(item::table)
        .values(&new_items)
        .execute(conn)
        .expect("Error saving new post");
}

pub fn fetch_items_task(conn: impl Deref<Target=PgConnection> + Send)
-> impl Future<Item=(), Error=()> + Send {
    let feeds = data::load_feeds(&conn)
        .expect("Error loading feeds");

    fetch_feeds(&feeds)
        .then(|result| -> Result<_, ()> {
            Ok(result)
        })
        .collect()
        .map(move |responses| {
            let iter = feeds.iter().zip(responses);
            insert_new_feed_items(iter, &conn)
        })
}

pub fn handle_api_request(request: &ApiRequest, connection: &PgConnection)
-> ApiResponse {
    let payload = match request.req_type {
        ApiRequestType::Groups => load_groups(connection),
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
        ApiRequestType::MarkItemSaved(id) => update_item_saved(id, true, connection),
        ApiRequestType::MarkItemUnsaved(id) => update_item_saved(id, false, connection),
        _ => ApiResponsePayload::None {},
    };
    ApiResponse {
        api_version: 1,
        auth: true,
        last_refreshed_on_time: None,
        payload: payload,
    }
}
