use std::error::Error;
use std::fmt;
use std::slice;

use atom_syndication as atom;
use rss;

use super::entry::Entry;

pub enum Feed {
    Rss(rss::Channel),
    Atom(atom::Feed),
}

impl Feed {
    pub fn parse(source: &[u8]) -> Result<Feed, FeedParseError> {
        match rss::Channel::read_from(source) {
            Ok(channel) => Ok(Feed::Rss(channel)),
            Err(rss::Error::InvalidStartTag) => {
                atom::Feed::read_from(source)
                    .map(Feed::Atom)
                    .map_err(FeedParseError::Atom)
            }
            Err(err) => Err(FeedParseError::Rss(err)),
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Feed::Rss(channel) => channel.title(),
            Feed::Atom(feed) => feed.title(),
        }.trim()
    }

    pub fn site_url(&self) -> Option<&str> {
        match self {
            Feed::Rss(channel) => Some(channel.link()),
            Feed::Atom(feed) => {
                feed.links()
                    .iter().filter(|link| link.rel() == "alternate").next()
                    .or(feed.links().first())
                    .map(|link| link.href())
            }
        }.map(str::trim)
    }

    pub fn len(&self) -> usize {
        match self {
            Feed::Rss(channel) => channel.items().len(),
            Feed::Atom(feed) => feed.entries().len(),
        }
    }

    pub fn entries<'a>(&'a self) -> impl Iterator<Item=Entry> + 'a {
        match *self {
            Feed::Rss(ref channel) => {
                Entries::Rss(channel.items().iter())
            }
            Feed::Atom(ref feed) => {
                Entries::Atom(feed.entries().iter())
            }
        }
    }
}

enum Entries<'a> {
    Rss(slice::Iter<'a, rss::Item>),
    Atom(slice::Iter<'a, atom::Entry>),
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        match self {
            Entries::Rss(items) => {
                items.next().map(Entry::from_rss)
            }
            Entries::Atom(entries) => {
                entries.next().map(Entry::from_atom)
            }
        }
    }
}

#[derive(Debug)]
pub enum FeedParseError {
    Rss(rss::Error),
    Atom(atom::Error),
}

impl fmt::Display for FeedParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FeedParseError::Rss(err) => fmt::Display::fmt(err, f),
            FeedParseError::Atom(err) => fmt::Display::fmt(err, f),
        }
    }
}


impl Error for FeedParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FeedParseError::Rss(err) => Some(err),
            FeedParseError::Atom(err) => Some(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use super::Feed;

    static RSS_STR: &'static str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>TechCrunch</title>
    <link>http://techcrunch.com</link>
    <description>The latest technology news and information on startups</description>
    <item>
      <title>Ford hires Elon Musk as CEO</title>
      <pubDate>01 Apr 2019 07:30:00 GMT</pubDate>
      <description>In an unprecedented move, Ford hires Elon Musk.</description>
    </item>
  </channel>
</rss>
"#;

    static ATOM_STR: &'static str = r#"
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <id>urn:uuid:b3420f84-6bdf-4f46-a225-f1b9a14703b6</id>
  <title>TechCrunch</title>
  <updated>2019-04-01T07:30:00Z</updated>
  <entry>
    <id>urn:uuid:4ae8550b-2987-49fa-9f8c-54c180c418ac</id>
    <title>Ford hires Elon Musk as CEO</title>
    <updated>2019-04-01T07:30:00Z</updated>
  </entry>
</feed>
"#;

    #[test]
    fn test_rss_stream() {
        let feed = Feed::parse(RSS_STR.as_bytes()).unwrap();
        let mut entries = feed.entries();

        let entry = entries.next().unwrap();
        assert_eq!(entry.title, "Ford hires Elon Musk as CEO");
        assert_eq!(entry.content, "In an unprecedented move, Ford hires Elon Musk.");
        let expected_date = Utc.with_ymd_and_hms(2019, 4, 1, 7, 30, 0).unwrap();
        assert_eq!(entry.published.unwrap(), expected_date);

        assert!(entries.next().is_none());
    }

    #[test]
    fn test_atom_stream() {
        let feed = Feed::parse(ATOM_STR.as_bytes()).unwrap();
        let mut entries = feed.entries();

        let entry = entries.next().unwrap();
        assert_eq!(entry.title, "Ford hires Elon Musk as CEO");
        assert_eq!(entry.id.unwrap(), "urn:uuid:4ae8550b-2987-49fa-9f8c-54c180c418ac");
        let expected_date = Utc.with_ymd_and_hms(2019, 4, 1, 7, 30, 0).unwrap();
        assert_eq!(entry.published.unwrap(), expected_date);

        assert!(entries.next().is_none());
    }
}
