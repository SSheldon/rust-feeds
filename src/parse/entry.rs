use atom_syndication::{Entry as AtomEntry};
use chrono::{DateTime, FixedOffset};
use rss::{Item as RssItem};
use url::Url;

use crate::item_identity::ItemIdentifier;

pub struct Entry {
    pub title: String,
    pub content: String,
    pub link: Option<String>,
    pub published: Option<DateTime<FixedOffset>>,
    pub author: Option<String>,
    pub guid: Option<String>,
}

impl Entry {
    pub fn from_ref(entry_ref: EntryRef) -> Entry {
        Entry {
            title: entry_ref.title().to_owned(),
            content: entry_ref.content().to_owned(),
            link: entry_ref.link().map(str::to_owned),
            published: entry_ref.published(),
            author: entry_ref.author().map(str::to_owned),
            guid: entry_ref.guid().map(str::to_owned),
        }
    }

    pub fn clear_redundant_guid(&mut self) {
        if self.guid == self.link {
            self.guid = None;
        }
    }

    pub(crate) fn expand_link(&mut self, base_url: &Url) {
        let link_url = self.link.as_ref()
            .and_then(|link| base_url.join(link).ok());

        self.link = link_url.map(Into::into).or(self.link.take());
    }

    pub fn identifier(&self) -> Option<ItemIdentifier> {
        ItemIdentifier::new(self.link.as_deref())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EntryRef<'a> {
    Rss(&'a RssItem),
    Atom(&'a AtomEntry),
}

impl<'a> EntryRef<'a> {
    pub fn title(self) -> &'a str {
        match self {
            Self::Rss(item) => item.title().unwrap_or(""),
            Self::Atom(entry) => entry.title(),
        }.trim()
    }

    pub fn content(self) -> &'a str {
        match self {
            Self::Rss(item) => {
                item.content()
                    .or(item.description())
            }
            Self::Atom(entry) => {
                entry.content()
                    .and_then(|content| content.value())
                    .or(entry.summary().map(|summary| summary.as_str()))
            }
        }.unwrap_or("").trim()
    }

    pub fn link(self) -> Option<&'a str> {
        match self {
            Self::Rss(item) => {
                item.extensions()
                    .get("feedburner")
                    .and_then(|ext| ext.get("origLink"))
                    .and_then(|exts| exts.first())
                    .and_then(|ext| ext.value())
                    .or(item.link())
                    .or_else(|| {
                        item.guid().and_then(|id| {
                            if id.is_permalink() { Some(id.value()) }
                            else { None }
                        })
                    })
            }
            Self::Atom(entry) => {
                entry.links()
                    .iter().filter(|link| link.rel() == "alternate").next()
                    .or(entry.links().first())
                    .map(|link| link.href())
            }
        }
    }

    pub fn published(self) -> Option<DateTime<FixedOffset>> {
        match self {
            Self::Rss(item) => {
                item.pub_date()
                    .and_then(|s| DateTime::parse_from_rfc2822(s).ok())
            }
            Self::Atom(entry) => {
                Some(*entry.published().unwrap_or(entry.updated()))
            }
        }
    }

    pub fn author(self) -> Option<&'a str> {
        match self {
            Self::Rss(item) => {
                item.author()
                    .or_else(|| {
                        item.dublin_core_ext()
                            .and_then(|ext| ext.creators().first())
                            .map(String::as_str)
                    })
            }
            Self::Atom(entry) => {
                entry.authors()
                    .first()
                    .map(|author| author.name())
            }
        }
    }

    pub fn guid(self) -> Option<&'a str> {
        match self {
            Self::Rss(item) => item.guid().map(|id| id.value()),
            Self::Atom(entry) => Some(entry.id()),
        }
    }
}
