use std::ascii::AsciiExt;
use std::borrow::Cow;

use atom_syndication::{Content, Entry as AtomEntry};
use atom_syndication::FromXml;
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
            EntryKind::Rss(ref item) =>
                item.description.as_ref().map(|s| Cow::from(&**s)),
            EntryKind::Atom(ref entry) =>
                entry.content.as_ref().map(cow_from_content),
        }
    }
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
