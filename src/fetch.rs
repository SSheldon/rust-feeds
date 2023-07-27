use std::error::Error as StdError;

use bytes::Bytes;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use futures::future;
use reqwest;
use reqwest::Client;
use url::Url;

use crate::data;
use crate::error::Error;
use crate::models::feed::{Feed, NewFeed};
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
        author: entry.author.as_deref(),
    }
}

fn parse_new_entries(
    response: Result<Bytes, reqwest::Error>,
    feed: &Feed,
    conn: &mut PgConnection,
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

    let base_url = parsed_feed.site_url().map(Url::parse)
        .unwrap_or_else(|| Url::parse(&feed.url));
    let base_url = match base_url {
        Ok(base_url) => base_url,
        Err(err) => {
            print!("Error parsing base url for {}: {}", feed.url, err);
            return Ok(entries);
        }
    };

    let parsed_entries: Vec<_> = parsed_feed.entries()
        // Currently we require entries to have links
        .filter(|entry| entry.link().is_some())
        .map(|entry_ref| {
            let mut entry = Entry::from_ref(entry_ref);
            // Some bad feeds use relative links...
            entry.expand_link(&base_url);
            entry
        })
        .collect();

    let latest_seen_urls: Vec<String> = vec![];

    if !latest_seen_urls.is_empty() {
        // If this feed has more entries than the 10 latest we guarantee are not pruned,
        // some of them may be ones that we already saw but have since pruned.
        // Find the last entry that we have seen and assume anything after was seen.
        let maybe_unseen_count = if parsed_entries.len() > latest_seen_urls.len() {
            parsed_entries.iter()
                .rposition(|entry| entry.link_in(&latest_seen_urls))
        } else {
            None
        }.unwrap_or(parsed_entries.len());

        let maybe_unseen_entries = parsed_entries.into_iter()
            .take(maybe_unseen_count)
            .filter(|entry| !entry.link_in(&latest_seen_urls));

        for entry in maybe_unseen_entries {
            let exists = data::item_already_exists(entry.link.as_ref().unwrap(), feed, conn)
                .map_err(fill_err!("Error querying if item exists"))?;

            if !exists {
                entries.push(entry);
            }
        }
    } else {
        entries = parsed_entries;
    }

    println!("Found {} new items of {} for {}",
        entries.len(), parsed_feed.len(), feed.url);
    Ok(entries)
}

async fn fetch_feed(url: &str, client: &Client)
-> Result<Bytes, reqwest::Error> {
    println!("Fetching items from {}...", url);
    let response = client.get(url)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 Gecko")
        .send()
        .await?;
    response.bytes().await
}

fn insert_items<'a>(
    iter: impl Iterator<Item=(&'a Feed, &'a Entry)>,
    conn: &'a mut PgConnection,
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

pub async fn fetch_items(conn: &mut PgConnection) -> DataResult<()> {
    let feeds = data::load_feeds(conn)
        .map_err(fill_err!("Error loading feeds"))?;
    let client = Client::new();

    for feeds in feeds.chunks(10) {
        let responses = feeds.iter().map(|feed| fetch_feed(&feed.url, &client));
        let responses = future::join_all(responses).await;

        let new_entries: Vec<_> = feeds.iter()
            .zip(responses)
            .map(|(feed, response)| parse_new_entries(response, feed, conn))
            .collect::<Result<_, _>>()?;

        let iter = feeds.iter()
            .zip(&new_entries)
            .flat_map(|(feed, entries)| {
                // Reverse order so older entries get inserted first
                entries.iter().rev().map(move |entry| (feed, entry))
            });
        insert_items(iter, conn)?;
    }

    Ok(())
}

fn insert_feed(feed: &ParsedFeed, url: &str, conn: &mut PgConnection)
-> DataResult<Feed> {
    use crate::schema::feed;

    let new_feed = NewFeed {
        url,
        title: feed.title(),
        site_url: feed.site_url(),
    };

    diesel::insert_into(feed::table)
        .values(&new_feed)
        .get_result(conn)
        .map_err(fill_err!("Error inserting new feed"))
}

pub async fn subscribe(url: &str, conn: &mut PgConnection)
-> Result<(), Box<dyn StdError + 'static>> {
    let client = Client::new();
    let response = fetch_feed(url, &client).await
        .map_err(fill_err!("Error fetching feed"))?;

    let parsed_feed = ParsedFeed::parse(&response)
        .map_err(fill_err!("Error parsing feed"))?;

    let feed = insert_feed(&parsed_feed, url, conn)?;

    let entries: Vec<_> = parsed_feed.entries().map(Entry::from_ref).collect();
    println!("Found {} items", entries.len());
    let iter = entries.iter().rev().map(|entry| (&feed, entry));
    insert_items(iter, conn)?;

    Ok(())
}
