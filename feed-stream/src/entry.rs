use std::borrow::Cow;

use atom_syndication::{Content, Entry as AtomEntry, Link, Person};
use atom_syndication::FromXml;
use chrono::{DateTime, FixedOffset};
use rss::{Guid, Item as RssItem};
use rss::ViaXml;
use xml::Element;

use parser::FeedParseError;

enum EntryKind {
    Rss(RssItem),
    Atom(AtomEntry),
}

pub struct Entry {
    kind: EntryKind,
}

impl Entry {
    pub fn title(&self) -> &str {
        match self.kind {
            EntryKind::Rss(ref item) => item.title.as_ref().map_or("", |s| &**s),
            EntryKind::Atom(ref entry) => &entry.title,
        }
    }

    pub fn content(&self) -> Option<Cow<str>> {
        fn cow_from_content(content: &Content) -> Cow<str> {
            match content {
                &Content::Text(ref s) => Cow::from(&**s),
                &Content::Html(ref s) => Cow::from(&**s),
                &Content::Xhtml(ref e) => Cow::from(e.to_string()),
            }
        }

        match self.kind {
            EntryKind::Rss(ref item) => maybe_cow_from_str(&item.description),
            EntryKind::Atom(ref entry) =>
                entry.content.as_ref().map(cow_from_content)
                    .or_else(|| maybe_cow_from_str(&entry.summary)),
        }
    }

    pub fn link(&self) -> Option<&str> {
        fn maybe_guid_link(id: &Guid) -> Option<&str> {
            if id.is_perma_link { Some(&id.value) } else { None }
        }

        fn is_alt_link(link: &Link) -> bool {
            link.rel.as_ref().map_or(true, |rel| rel == "alternate")
        }

        match self.kind {
            EntryKind::Rss(ref item) =>
                item.link.as_ref().map(|s| &**s)
                    .or_else(|| item.guid.as_ref().and_then(maybe_guid_link)),
            EntryKind::Atom(ref entry) =>
                entry.links.iter().find(|&link| is_alt_link(link))
                    .or_else(|| entry.links.first())
                    .map(|link| &*link.href),
        }
    }

    pub fn published(&self) -> Option<DateTime<FixedOffset>> {
        match self.kind {
            EntryKind::Rss(ref item) =>
                item.pub_date.as_ref()
                    .and_then(|s| DateTime::parse_from_rfc2822(s).ok()),
            EntryKind::Atom(ref entry) =>
                entry.published.as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .or_else(|| DateTime::parse_from_rfc3339(&entry.updated).ok()),
        }
    }

    pub fn authors(&self) -> Vec<Cow<str>> {
        fn author_str(author: &Person) -> Cow<str> {
            match author.email {
                Some(ref email) =>
                    Cow::from(format!("{} ({})", email, author.name)),
                None => Cow::from(&*author.name),
            }
        }

        match self.kind {
            EntryKind::Rss(ref item) =>
                maybe_cow_from_str(&item.author)
                    .map_or(Vec::new(), |s| vec![s]),
            EntryKind::Atom(ref entry) =>
                entry.authors.iter().map(author_str).collect(),
        }
    }

    pub fn id(&self) -> Option<&str> {
        match self.kind {
            EntryKind::Rss(ref item) => item.guid.as_ref().map(|id| &*id.value),
            EntryKind::Atom(ref entry) => Some(&entry.id),
        }
    }
}


fn maybe_cow_from_str(s: &Option<String>) -> Option<Cow<str>> {
    s.as_ref().map(|s| Cow::from(&**s))
}

pub fn from_rss_item(elem: Element) -> Result<Entry, FeedParseError> {
    RssItem::from_xml(elem)
        .map(|item| Entry { kind: EntryKind::Rss(item) })
        .map_err(|e| FeedParseError::Rss(e))
}

pub fn from_atom_entry(elem: Element) -> Result<Entry, FeedParseError> {
    AtomEntry::from_xml(&elem)
        .map(|entry| Entry { kind: EntryKind::Atom(entry) })
        .map_err(|e| FeedParseError::Atom(e))
}
