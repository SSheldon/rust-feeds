use std::collections::HashMap;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use futures::{Future, Stream, future, stream};
use iter_read::IterRead;
use reqwest;
use reqwest::async::{Chunk, Client};

use feed_stream::{Entry, FeedParser};
use fever_api::{
    ApiKey, ApiRequest, ApiRequestType, ApiResponse, ApiResponsePayload, self,
};

use config::PgConnectionPool;
use data::{ItemsQuery, self};
use error::Error;
use models::feed::Feed as DbFeed;
use models::group::Group as DbGroup;
use models::item::{Item as DbItem, NewItem};

type DataError = Error<diesel::result::Error>;
type DataResult<T> = Result<T, DataError>;
type FetchError = Box<::std::error::Error + Send + Sync>;

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
        site_url: feed.site_url,
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

fn load_groups(conn: &PgConnection) -> DataResult<ApiResponsePayload> {
    let groups = data::load_groups(conn)
        .map_err(fill_err!("Error loading groups"))?
        .into_iter()
        .map(format_group)
        .collect();

    let feed_groups = data::load_feed_groups(conn)
        .map_err(fill_err!("Error loading feeds"))?;
    let feeds_groups = format_feeds_groups(feed_groups.into_iter());

    Ok(ApiResponsePayload::Groups { groups, feeds_groups })
}

fn load_feeds(conn: &PgConnection) -> DataResult<ApiResponsePayload> {
    let feeds = data::load_feeds(conn)
        .map_err(fill_err!("Error loading feeds"))?;

    let feeds_groups = {
        let feed_groups = feeds.iter()
            .map(|feed| (feed.id, feed.group_id));

        format_feeds_groups(feed_groups)
    };

    let feeds = feeds
        .into_iter()
        .map(format_feed)
        .collect();

    Ok(ApiResponsePayload::Feeds {
        feeds: feeds,
        feeds_groups: feeds_groups,
    })
}

fn load_items(query: ItemsQuery, conn: &PgConnection)
-> DataResult<ApiResponsePayload> {
    let items = data::load_items(query, conn)
        .map_err(fill_err!("Error loading items"))?
        .into_iter()
        .map(format_item)
        .collect();
    let total_items = data::count_items(conn)
        .map_err(fill_err!("Error counting items"))?;

    Ok(ApiResponsePayload::Items {
        items: items,
        total_items: total_items,
    })
}

fn load_unread_item_ids(conn: &PgConnection) -> DataResult<ApiResponsePayload> {
    let ids = data::load_unread_item_ids(conn)
        .map_err(fill_err!("Error loading unread item ids"))?
        .into_iter()
        .map(|i| i as u32)
        .collect();

    Ok(ApiResponsePayload::UnreadItems {
        unread_item_ids: ids,
    })
}

fn load_saved_item_ids(conn: &PgConnection) -> DataResult<ApiResponsePayload> {
    let ids = data::load_saved_item_ids(conn)
        .map_err(fill_err!("Error loading saved item ids"))?
        .into_iter()
        .map(|i| i as u32)
        .collect();

    Ok(ApiResponsePayload::SavedItems {
        saved_item_ids: ids,
    })
}

fn update_item_read(id: u32, is_read: bool, conn: &PgConnection)
-> DataResult<ApiResponsePayload> {
    use schema::item;

    diesel::update(item::table.find(id as i32))
        .set(item::is_read.eq(is_read))
        .execute(conn)
        .map_err(fill_err!("Error updating item is_read"))?;

    load_unread_item_ids(conn)
}

fn update_item_saved(id: u32, is_saved: bool, conn: &PgConnection)
-> DataResult<ApiResponsePayload> {
    use schema::item;

    diesel::update(item::table.find(id as i32))
        .set(item::is_saved.eq(is_saved))
        .execute(conn)
        .map_err(fill_err!("Error updating item is_saved"))?;

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
    conn: &PgConnection,
) -> DataResult<Vec<Entry>> {
    let chunks = response.iter().map(|chunk| -> &[u8] { &chunk });
    let parser = FeedParser::new(IterRead::new(chunks));

    let mut entries = Vec::new();
    for entry in parser {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                println!("Error parsing {}: {}", feed.url, err);
                break;
            }
        };
        let exists = match entry.link.as_ref() {
            Some(link) => {
                data::item_already_exists(link, feed, conn)
                    .map_err(fill_err!("Error querying if item exists"))?
            }
            // Currently we require entries to have links
            None => continue,
        };
        // If we've reached an item that we've already seen, stop parsing
        if exists {
            break;
        }
        entries.push(entry);
    }
    Ok(entries)
}

fn fetch_feed(feed: &DbFeed, client: &Client)
-> impl Future<Item=Vec<Chunk>, Error=reqwest::Error> + 'static {
    println!("Fetching items from {}...", feed.url);
    client.get(&feed.url)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 Gecko")
        .send()
        .and_then(|response| {
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
) -> DataResult<()> {
    use schema::item;

    let feed_entries: Vec<_> = iter
        .filter_map(|(feed, response)| {
            match response {
                Ok(response) => Some((feed, response)),
                Err(err) => {
                    println!("Error fetching from {}: {}", feed.url, err);
                    None
                }
            }
        })
        .map(|(feed, response)| {
            parse_new_entries(&response, feed, conn)
                .map(|entries| {
                    println!("Found {} new items for {}", entries.len(), feed.url);
                    (feed, entries)
                })
        })
        .collect::<Result<_, _>>()?;

    let mut new_items = Vec::new();
    for &(feed, ref entries) in feed_entries.iter() {
        for entry in entries.iter().rev() {
            new_items.push(item_to_insert_for_entry(entry, feed));
        }
    }

    diesel::insert_into(item::table)
        .values(&new_items)
        .execute(conn)
        .map_err(fill_err!("Error saving new items"))?;

    Ok(())
}

fn fetch_feeds_task(feeds: Vec<DbFeed>, pool: PgConnectionPool)
-> impl Future<Item=(), Error=FetchError> + Send {
    fetch_feeds(&feeds)
        .then(|result| -> Result<_, FetchError> {
            Ok(result)
        })
        .collect()
        .and_then(move |responses| {
            let conn = pool.get()
                .map_err(fill_err!("Error getting connection from pool"))?;
            let iter = feeds.iter().zip(responses);
            insert_new_feed_items(iter, &conn)?;
            Ok(())
        })
}

fn chunk<T>(v: Vec<T>, size: usize) -> Vec<Vec<T>> {
    if v.is_empty() {
        return Vec::new();
    }

    let num_chunks = v.len() / size + if v.len() % size != 0 {1} else {0};
    let mut elems = v.into_iter();
    (0..num_chunks)
        .map(|_| elems.by_ref().take(size).collect())
        .collect()
}

pub fn fetch_items_task(pool: PgConnectionPool)
-> impl Future<Item=(), Error=FetchError> + Send {
    let pool2 = pool.clone();
    future::lazy(move || {
        let conn = pool.get()
            .map_err(fill_err!("Error getting connection from pool"))?;
        let feeds = data::load_feeds(&conn)
            .map_err(fill_err!("Error loading feeds"))?;
        Ok(feeds)
    }).and_then(|feeds| {
        let feeds = chunk(feeds, 10);
        stream::iter_ok(feeds)
            .and_then(move |feeds| fetch_feeds_task(feeds, pool2.clone()))
            .collect()
            .map(|_| ())
    })
}

pub fn handle_api_request(
    request: &ApiRequest,
    expected_key: Option<&ApiKey>,
    conn: &PgConnection,
) -> DataResult<ApiResponse> {
    let mut response = ApiResponse {
        api_version: 1,
        auth: false,
        last_refreshed_on_time: None,
        payload: ApiResponsePayload::None {},
    };

    if !expected_key.map_or(true, |key| request.api_key == *key) {
        return Ok(response);
    }
    response.auth = true;

    response.payload = match request.req_type {
        ApiRequestType::Groups => load_groups(conn)?,
        ApiRequestType::Feeds => load_feeds(conn)?,
        ApiRequestType::LatestItems => {
            load_items(ItemsQuery::Latest, conn)?
        },
        ApiRequestType::ItemsBefore(id) => {
            load_items(ItemsQuery::Before(id as i32), conn)?
        },
        ApiRequestType::ItemsSince(id) => {
            load_items(ItemsQuery::After(id as i32), conn)?
        },
        ApiRequestType::Items(ref ids) => {
            let ids: Vec<_> = ids.iter().map(|&i| i as i32).collect();
            load_items(ItemsQuery::ForIds(&ids), conn)?
        }
        ApiRequestType::UnreadItems => load_unread_item_ids(conn)?,
        ApiRequestType::MarkItemRead(id) => update_item_read(id, true, conn)?,
        ApiRequestType::MarkItemUnread(id) => update_item_read(id, false, conn)?,
        ApiRequestType::MarkItemSaved(id) => update_item_saved(id, true, conn)?,
        ApiRequestType::MarkItemUnsaved(id) => update_item_saved(id, false, conn)?,
        _ => ApiResponsePayload::None {},
    };

    Ok(response)
}
