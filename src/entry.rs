use xml::Element;

use {Category, Link, NS, Person, Source};
use author::Author;
use contributor::Contributor;
use utils::{ElementUtils, Flip, FromXml, ToXml};


/// [The Atom Syndication Format § The "atom:entry" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.1.2)
///
/// # Examples
///
/// ```
/// use atom_syndication::Entry;
///
/// let entry = Entry {
///     id: String::from("9dd22af1-7298-4ca6-af40-6a44bae3726f"),
///     title: String::from("A blog post title"),
///     updated: String::from("2015-05-11T21:30:54Z"),
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Default)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub updated: String,
    pub published: Option<String>,
    pub source: Option<Source>,
    pub links: Vec<Link>,
    pub categories: Vec<Category>,
    pub authors: Vec<Person>,
    pub contributors: Vec<Person>,
    pub summary: Option<String>,
    pub content: Option<String>,
}


impl ToXml for Entry {
    fn to_xml(&self) -> Element {
        let mut entry = Element::new("entry".to_string(), Some(NS.to_string()), vec![]);

        entry.tag_with_text("id", &self.id);
        entry.tag_with_text("title", &self.title);
        entry.tag_with_text("updated", &self.updated);

        entry.tag_with_optional_text("published", &self.published);

        if let Some(ref s) = self.source {
            entry.tag(s.to_xml());
        }

        for link in &self.links {
            entry.tag(link.to_xml());
        }

        for category in &self.categories {
            entry.tag(category.to_xml());
        }

        for person in &self.authors {
            entry.tag(Author(person).to_xml());
        }

        for person in &self.contributors {
            entry.tag(Contributor(person).to_xml());
        }

        entry.tag_with_optional_text("summary", &self.summary);
        entry.tag_with_optional_text("content", &self.content);

        entry
    }
}


impl FromXml for Entry {
    fn from_xml(elem: &Element) -> Result<Self, &'static str> {
        let id = match elem.get_child("id", Some(NS)) {
            Some(elem) => elem.content_str(),
            None => return Err("<entry> is missing required <id> element"),
        };

        let title = match elem.get_child("title", Some(NS)) {
            Some(elem) => elem.content_str(),
            None => return Err("<entry> is missing required <title> element"),
        };

        let updated = match elem.get_child("updated", Some(NS)) {
            Some(elem) => elem.content_str(),
            None => return Err("<entry> is missing required <updated> element"),
        };

        let source = try!(elem.get_child("source", Some(NS))
            .map(|e| FromXml::from_xml(e)).flip());

        let links = try!(elem.get_children("link", Some(NS))
            .map(|e| FromXml::from_xml(e))
            .collect());

        let categories = try!(elem.get_children("category", Some(NS))
            .map(|e| FromXml::from_xml(e))
            .collect());

        let authors = try!(elem.get_children("author", Some(NS))
            .map(|e| FromXml::from_xml(e).map(|Author(person)| person))
            .collect());

        let contributors = try!(elem.get_children("contributor", Some(NS))
            .map(|e| FromXml::from_xml(e).map(|Contributor(person)| person))
            .collect());

        let published = elem.get_child("published", Some(NS)).map(Element::content_str);
        let summary = elem.get_child("summary", Some(NS)).map(Element::content_str);
        let content = elem.get_child("content", Some(NS)).map(Element::content_str);

        Ok(Entry {
            id: id,
            title: title,
            updated: updated,
            published: published,
            source: source,
            links: links,
            categories: categories,
            authors: authors,
            contributors: contributors,
            summary: summary,
            content: content,
        })
    }
}
