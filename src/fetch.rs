use bytes::Bytes;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use futures::future;
use reqwest;
use reqwest::Client;

use feed_stream::{Entry, FeedParser};

use crate::config::PgConnectionPool;
use crate::data;
use crate::error::Error;
use crate::models::feed::Feed;
use crate::models::item::NewItem;

type DataResult<T> = Result<T, Error>;

fn item_to_insert_for_entry<'a>(entry: &'a Entry, feed: &Feed) -> NewItem<'a> {
    NewItem {
        url: entry.link.as_ref().unwrap(),
        title: &entry.title,
        content: &entry.content,
        published: entry.published.as_ref().map(|d| d.naive_utc()),
        feed_id: feed.id,
    }
}

fn parse_new_entries(
    response: &[u8],
    feed: &Feed,
    conn: &PgConnection,
) -> DataResult<Vec<Entry>> {
    let parser = FeedParser::new(response);

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

async fn fetch_feed(feed: &Feed, client: &Client)
-> Result<Bytes, reqwest::Error> {
    println!("Fetching items from {}...", feed.url);
    let response = client.get(&feed.url)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 Gecko")
        .send()
        .await?;
    response.bytes().await
}

fn insert_new_feed_items<'a>(
    iter: impl Iterator<Item=(&'a Feed, Result<Bytes, reqwest::Error>)>,
    conn: &'a PgConnection,
) -> DataResult<()> {
    use crate::schema::item;

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

pub async fn fetch_items_task(pool: PgConnectionPool) -> Result<(), Error> {
    let conn = pool.get()
        .map_err(fill_err!("Error getting connection from pool"))?;
    let feeds = data::load_feeds(&conn)
        .map_err(fill_err!("Error loading feeds"))?;
    let client = Client::new();

    for feeds in feeds.chunks(10) {
        let responses = feeds.iter().map(|feed| fetch_feed(feed, &client));
        let responses = future::join_all(responses).await;
        let iter = feeds.iter().zip(responses);
        insert_new_feed_items(iter, &conn)?;
    }

    Ok(())
}
