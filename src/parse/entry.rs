use atom_syndication::{Entry as AtomEntry};
use chrono::{DateTime, FixedOffset};
use rss::{Item as RssItem};

pub struct Entry {
    pub title: String,
    pub content: String,
    pub link: Option<String>,
    pub published: Option<DateTime<FixedOffset>>,
    pub authors: Vec<String>,
    pub id: Option<String>,
}

impl Entry {
    pub(super) fn from_rss(item: &RssItem) -> Entry {
        let title = item.title()
            .map(str::trim)
            .map(str::to_owned)
            .unwrap_or(String::new());

        let content = item.content()
            .or(item.description())
            .map(str::trim)
            .map(str::to_owned)
            .unwrap_or(String::new());

        let orig_link = item.extensions()
            .get("feedburner")
            .and_then(|ext| ext.get("origLink"))
            .and_then(|exts| exts.first())
            .and_then(|ext| ext.value());

        let link = orig_link
            .or(item.link())
            .or_else(|| {
                item.guid().and_then(|id| {
                    if id.is_permalink() { Some(id.value()) }
                    else { None }
                })
            })
            .map(str::to_owned);

        let published = item.pub_date()
            .and_then(|s| DateTime::parse_from_rfc2822(s).ok());

        let authors = item.author()
            .map(str::to_owned)
            .map_or(Vec::new(), |s| vec![s]);

        let id = item.guid()
            .map(|id| id.value().to_owned());

        Entry { title, content, link, published, authors, id }
    }

    pub(super) fn from_atom(entry: &AtomEntry) -> Entry {
        let title = entry.title()
            .trim()
            .to_owned();

        let content = entry.content()
            .and_then(|content| content.value())
            .or(entry.summary())
            .map(str::trim)
            .map(str::to_owned)
            .unwrap_or(String::new());

        let link = entry.links()
            .iter().filter(|link| link.rel() == "alternate").next()
            .or(entry.links().first())
            .map(|link| link.href())
            .map(str::to_owned);

        let published = entry.published()
            .unwrap_or(entry.updated())
            .clone();
        let published = Some(published);

        let authors = entry.authors().iter()
            .map(|author| {
                if let Some(email) = author.email() {
                    format!("{} ({})", author.name(), email)
                } else {
                    author.name().to_owned()
                }
            })
            .collect();

        let id = entry.id().to_owned();
        let id = Some(id);

        Entry { title, content, link, published, authors, id }
    }
}
