use bytes::Bytes;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use futures::future;
use reqwest;
use reqwest::Client;

use crate::config::MaybePooled;
use crate::data;
use crate::error::Error;
use crate::models::feed::Feed;
use crate::models::item::NewItem;
use crate::parse::{Entry, Feed as ParsedFeed};

type DataResult<T> = Result<T, Error<diesel::result::Error>>;

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
    response: Result<Bytes, reqwest::Error>,
    feed: &Feed,
    conn: &PgConnection,
) -> DataResult<Vec<Entry>> {
    let mut entries = Vec::new();

    let response = match response {
        Ok(response) => response,
        Err(err) => {
            println!("Error fetching from {}: {}", feed.url, err);
            return Ok(entries);
        }
    };

    let parsed_feed = match ParsedFeed::parse(&response) {
        Ok(parsed_feed) => parsed_feed,
        Err(err) => {
            println!("Error parsing {}: {}", feed.url, err);
            return Ok(entries);
        }
    };

    for entry in parsed_feed.entries() {
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

    println!("Found {} new items for {}", entries.len(), feed.url);
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

fn insert_items<'a>(
    iter: impl Iterator<Item=(&'a Feed, &'a Entry)>,
    conn: &'a PgConnection,
) -> DataResult<()> {
    use crate::schema::item;

    let new_items: Vec<_> = iter
        .map(|(feed, entry)| item_to_insert_for_entry(entry, feed))
        .collect();

    diesel::insert_into(item::table)
        .values(&new_items)
        .execute(conn)
        .map_err(fill_err!("Error saving new items"))?;

    Ok(())
}

pub async fn fetch_items(conn: MaybePooled<PgConnection>) -> DataResult<()> {
    let feeds = data::load_feeds(&conn)
        .map_err(fill_err!("Error loading feeds"))?;
    let client = Client::new();

    for feeds in feeds.chunks(10) {
        let responses = feeds.iter().map(|feed| fetch_feed(feed, &client));
        let responses = future::join_all(responses).await;

        let new_entries: Vec<_> = feeds.iter()
            .zip(responses)
            .map(|(feed, response)| parse_new_entries(response, feed, &conn))
            .collect::<Result<_, _>>()?;

        let iter = feeds.iter()
            .zip(&new_entries)
            .flat_map(|(feed, entries)| {
                // Reverse order so older entries get inserted first
                entries.iter().rev().map(move |entry| (feed, entry))
            });
        insert_items(iter, &conn)?;
    }

    Ok(())
}
