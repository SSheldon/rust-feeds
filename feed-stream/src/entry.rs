use std::ascii::AsciiExt;
use std::borrow::Cow;

use atom_syndication::{Content, Entry as AtomEntry, Link, Person};
use atom_syndication::FromXml;
use chrono::{DateTime, FixedOffset};
use rss::Item as RssItem;
use rss::ViaXml;
use xml::Element;

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
        fn is_alt_link(link: &Link) -> bool {
            link.rel.as_ref().map_or(true, |rel| rel == "alternate")
        }

        match self.kind {
            EntryKind::Rss(ref item) => item.link.as_ref().map(|s| &**s),
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
}


fn maybe_cow_from_str(s: &Option<String>) -> Option<Cow<str>> {
    s.as_ref().map(|s| Cow::from(&**s))
}

pub fn from_xml(elem: Element) -> Option<Entry> {
    if elem.name.eq_ignore_ascii_case("item") {
        RssItem::from_xml(elem).ok().map(|item| {
            Entry { kind: EntryKind::Rss(item) }
        })
    } else if elem.name.eq_ignore_ascii_case("entry") {
        AtomEntry::from_xml(&elem).ok().map(|entry| {
            Entry { kind: EntryKind::Atom(entry) }
        })
    } else {
        None
    }
}
