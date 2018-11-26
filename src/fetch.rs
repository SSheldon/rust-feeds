use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use futures::{Future, Stream, future, stream};
use iter_read::IterRead;
use reqwest;
use reqwest::async::{Chunk, Client};

use feed_stream::{Entry, FeedParser};

use config::PgConnectionPool;
use data;
use error::Error;
use models::feed::Feed as DbFeed;
use models::item::NewItem;

type DataError = Error<diesel::result::Error>;
type DataResult<T> = Result<T, DataError>;
type FetchError = Box<::std::error::Error + Send + Sync>;

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
