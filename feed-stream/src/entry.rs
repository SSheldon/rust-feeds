use atom_syndication::{Content, Entry as AtomEntry, Link, Person};
use atom_syndication::FromXml;
use chrono::{DateTime, FixedOffset};
use rss::{Guid, Item as RssItem};
use rss::ViaXml;
use xml::Element;

use parser::FeedParseError;

const FEEDBURNER_NS: &str = "http://rssnamespace.org/feedburner/ext/1.0";
const CONTENT_NS: &str = "http://purl.org/rss/1.0/modules/content/";

pub struct Entry {
    pub title: String,
    pub content: String,
    pub link: Option<String>,
    pub published: Option<DateTime<FixedOffset>>,
    pub authors: Vec<String>,
    pub id: Option<String>,
}

impl Entry {
    pub(crate) fn from_rss_element(elem: Element)
    -> Result<Entry, FeedParseError> {
        let orig_link = elem
            .get_child("origLink", Some(FEEDBURNER_NS))
            .map(Element::content_str);

        let encoded_content = elem
            .get_child("encoded", Some(CONTENT_NS))
            .map(Element::content_str);

        let item = RssItem::from_xml(elem).map_err(FeedParseError::Rss)?;

        let mut entry = Entry::from_rss(item);
        // If there was an original link, use it instead
        entry.link = orig_link.or(entry.link);
        entry.content = encoded_content.unwrap_or(entry.content);
        Ok(entry)
    }

    fn from_rss(item: RssItem) -> Entry {
        fn maybe_guid_link(id: &Guid) -> Option<String> {
            if id.is_perma_link { Some(id.value.clone()) } else { None }
        }

        let RssItem { title, link, description, author, guid, pub_date, .. } = item;

        let link = link.or_else(|| guid.as_ref().and_then(maybe_guid_link));
        let published = pub_date.and_then(|s| DateTime::parse_from_rfc2822(&s).ok());
        let authors = author.map_or(Vec::new(), |s| vec![s]);
        let id = guid.map(|id| id.value);

        Entry {
            title: title.unwrap_or(String::new()),
            content: description.unwrap_or(String::new()),
            link: link,
            published: published,
            authors: authors,
            id: id,
        }
    }

    pub(crate) fn from_atom_element(elem: Element)
    -> Result<Entry, FeedParseError> {
        let entry = AtomEntry::from_xml(&elem).map_err(FeedParseError::Atom)?;

        let entry = Entry::from_atom(entry);
        Ok(entry)
    }

    fn from_atom(entry: AtomEntry) -> Entry {
        fn is_alt_link(link: &Link) -> bool {
            link.rel.as_ref().map_or(true, |rel| rel == "alternate")
        }

        fn author_string_from_person(author: Person) -> String {
            use std::fmt::Write;

            let Person { mut name, email, .. } = author;
            if let Some(email) = email {
                write!(&mut name, " ({})", email).unwrap();
            }
            name
        }

        let AtomEntry { id, title, updated, published, links, authors, summary, content, ..} = entry;

        let content = content.map(|content| match content {
            Content::Text(s) => s,
            Content::Html(s) => s,
            Content::Xhtml(e) => e.to_string(),
        }).or(summary).unwrap_or(String::new());

        let alt_link_pos = links.iter().position(is_alt_link);
        let link = if let Some(pos) = alt_link_pos {
            let mut links = links;
            Some(links.swap_remove(pos))
        } else {
            links.into_iter().next()
        };

        let published = published
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .or_else(|| DateTime::parse_from_rfc3339(&updated).ok());

        let authors = authors.into_iter()
            .map(author_string_from_person)
            .collect();

        Entry {
            title: title,
            content: content,
            link: link.map(|link| link.href),
            published: published,
            authors: authors,
            id: Some(id),
        }
    }
}
