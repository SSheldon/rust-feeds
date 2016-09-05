use std::str::FromStr;
use xml::{Element, ElementBuilder, Parser, Xml};

use {Category, Entry, Generator, Link, NS, Person};
use author::Author;
use contributor::Contributor;
use utils::{ElementUtils, Flip, ViaXml};


/// [The Atom Syndication Format § The "atom:feed" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.1.1)
///
/// # Examples
///
/// ```
/// use atom_syndication::Feed;
///
/// let feed = Feed {
///     id: String::from("6011425f-414d-4a17-84ba-b731c2bb1fc2"),
///     title: String::from("My Blog"),
///     updated: String::from("2015-05-11T21:30:54Z"),
///     entries: vec![],
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Default)]
pub struct Feed {
    pub id: String,
    pub title: String,
    pub updated: String,
    pub icon: Option<String>,
    pub logo: Option<String>,
    pub rights: Option<String>,
    pub subtitle: Option<String>,
    pub generator: Option<Generator>,
    pub links: Vec<Link>,
    pub categories: Vec<Category>,
    pub authors: Vec<Person>,
    pub contributors: Vec<Person>,
    pub entries: Vec<Entry>,
}

impl ViaXml for Feed {
    fn to_xml(&self) -> Element {
        let mut feed = Element::new("feed".to_string(), Some(NS.to_string()), vec![]);

        feed.tag_with_text("id", &self.id);
        feed.tag_with_text("title", &self.title);
        feed.tag_with_text("updated", &self.updated);

        feed.tag_with_optional_text("icon", &self.icon);
        feed.tag_with_optional_text("logo", &self.logo);
        feed.tag_with_optional_text("rights", &self.rights);
        feed.tag_with_optional_text("subtitle", &self.subtitle);

        if let Some(ref g) = self.generator {
            feed.tag(g.to_xml());
        }

        for link in &self.links {
            feed.tag(link.to_xml());
        }

        for category in &self.categories {
            feed.tag(category.to_xml());
        }

        for person in &self.authors {
            feed.tag(Author(person.clone()).to_xml());
        }

        for person in &self.contributors {
            feed.tag(Contributor(person.clone()).to_xml());
        }

        for entry in &self.entries {
            feed.tag(entry.to_xml());
        }

        feed
    }

    fn from_xml(elem: Element) -> Result<Self, &'static str> {
        let id = match elem.get_child("id", Some(NS)) {
            Some(elem) => elem.content_str(),
            None => return Err("<feed> is missing required <id> element"),
        };

        let title = match elem.get_child("title", Some(NS)) {
            Some(elem) => elem.content_str(),
            None => return Err("<feed> is missing required <title> element"),
        };

        let updated = match elem.get_child("updated", Some(NS)) {
            Some(elem) => elem.content_str(),
            None => return Err("<feed> is missing required <updated> element"),
        };

        let icon = elem.get_child("icon", Some(NS)).map(Element::content_str);
        let logo = elem.get_child("logo", Some(NS)).map(Element::content_str);
        let rights = elem.get_child("rights", Some(NS)).map(Element::content_str);
        let subtitle = elem.get_child("subtitle", Some(NS)).map(Element::content_str);

        let generator = try!(elem.get_child("generator", Some(NS))
            .map(|e| ViaXml::from_xml(e.clone())).flip());

        let links = try!(elem.get_children("link", Some(NS))
            .map(|e| ViaXml::from_xml(e.clone()))
            .collect());

        let categories = try!(elem.get_children("category", Some(NS))
            .map(|e| ViaXml::from_xml(e.clone()))
            .collect());

        let authors = try!(elem.get_children("author", Some(NS))
            .map(|e| ViaXml::from_xml(e.clone()).map(|Author(person)| person))
            .collect());

        let contributors = try!(elem.get_children("contributor", Some(NS))
            .map(|e| ViaXml::from_xml(e.clone()).map(|Contributor(person)| person))
            .collect());

        let entries = try!(elem.get_children("entry", Some(NS))
            .map(|e| ViaXml::from_xml(e.clone()))
            .collect());

        Ok(Feed {
            id: id,
            title: title,
            updated: updated,
            icon: icon,
            logo: logo,
            rights: rights,
            subtitle: subtitle,
            generator: generator,
            links: links,
            categories: categories,
            authors: authors,
            contributors: contributors,
            entries: entries,
        })
    }
}


impl FromStr for Feed {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new();
        parser.feed_str(s);

        let mut builder = ElementBuilder::new();

        for event in parser {
            if let Some(Ok(elem)) = builder.handle_event(event) {
                return ViaXml::from_xml(elem);
            }
        }

        Err("Atom read error")
    }
}

impl ToString for Feed {
    fn to_string(&self) -> String {
        let mut ret = format!("{}", Xml::PINode(r#"xml version="1.0" encoding="utf-8""#.to_string()));
        ret.push_str(&format!("{}", self.to_xml()));
        ret
    }
}
