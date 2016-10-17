use xml::Element;

use {Category, Content, Link, NS, Person, Source};
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
#[derive(Clone, Default, Debug, PartialEq)]
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
    pub content: Option<Content>,
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

        if let Some(ref content) = self.content {
            entry.tag(content.to_xml());
        }

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
        let content = try!(elem.get_child("content", Some(NS))
            .map(|e| FromXml::from_xml(e)).flip());

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


#[cfg(test)]
mod tests {
    use xml::{Element, Xml};

    use {Category, Content, Entry, Generator, Link, NS, Person, Source, XHTML_NS};
    use utils::{FromXml, ToXml};

    #[test]
    fn to_xml_minimal() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated></entry>");
    }

    #[test]
    fn to_xml_with_published() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            published: Some("2016-09-17T19:16:03Z".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><published>2016-09-17T19:16:03Z</published></entry>");
    }

    #[test]
    fn to_xml_with_source() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            source: Some(Source::default()),
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><source/></entry>");
    }

    #[test]
    fn to_xml_with_full_source() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            source: Some(Source {
                id: Some("http://example.com/feed.atom".to_string()),
                title: Some("Example Domain Updates".to_string()),
                updated: Some("2016-08-07T14:13:49Z".to_string()),
                icon: Some("http://example.com/icon.png".to_string()),
                logo: Some("http://example.com/logo.png".to_string()),
                rights: Some("All rights reversed.".to_string()),
                subtitle: Some("An example in the domain of Internet domains".to_string()),
                generator: Some(Generator {
                    name: "Atom Feed Generator Deluxe".to_string(),
                    ..Default::default()
                }),
                links: vec![
                    Link {
                        href: "http://example.com/".to_string(),
                        ..Default::default()
                    },
                ],
                categories: vec![
                    Category {
                        term: "announcements".to_string(),
                        ..Default::default()
                    },
                ],
                authors: vec![
                    Person {
                        name: "John Doe".to_string(),
                        ..Default::default()
                    },
                ],
                contributors: vec![
                    Person {
                        name: "Corey Farwell".to_string(),
                        ..Default::default()
                    },
                ],
            }),
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'>\
            <id>http://example.com/1</id>\
            <title>First!</title>\
            <updated>2016-09-17T19:18:32Z</updated>\
            <source>\
                <id>http://example.com/feed.atom</id>\
                <title>Example Domain Updates</title>\
                <updated>2016-08-07T14:13:49Z</updated>\
                <icon>http://example.com/icon.png</icon>\
                <logo>http://example.com/logo.png</logo>\
                <rights>All rights reversed.</rights>\
                <subtitle>An example in the domain of Internet domains</subtitle>\
                <generator>Atom Feed Generator Deluxe</generator>\
                <link href='http://example.com/'/>\
                <category term='announcements'/>\
                <author>\
                    <name>John Doe</name>\
                </author>\
                <contributor>\
                    <name>Corey Farwell</name>\
                </contributor>\
            </source>\
        </entry>");
    }

    #[test]
    fn to_xml_with_one_link() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><link href='http://example.com/'/></entry>");
    }

    #[test]
    fn to_xml_with_one_full_link() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    rel: Some("alternate".to_string()),
                    mediatype: Some("text/html".to_string()),
                    hreflang: Some("en-US".to_string()),
                    title: Some("Example Domain".to_string()),
                    length: Some("606".to_string()),
                },
            ],
            ..Default::default()
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = entry.to_xml();
        let mut expected_element = Element::new(
            "entry".to_string(),
            Some(NS.to_string()),
            vec![]);
        let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
        id_element.text("http://example.com/1".to_string());
        let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
        title_element.text("First!".to_string());
        let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
        updated_element.text("2016-09-17T19:18:32Z".to_string());
        let link_element = Element::new(
            "link".to_string(),
            Some(NS.to_string()),
            vec![
                ("href".to_string(), None, "http://example.com/".to_string()),
                ("rel".to_string(), None, "alternate".to_string()),
                ("type".to_string(), None, "text/html".to_string()),
                ("hreflang".to_string(), None, "en-US".to_string()),
                ("title".to_string(), None, "Example Domain".to_string()),
                ("length".to_string(), None, "606".to_string()),
            ]);
        expected_element
            .tag_stay(id_element)
            .tag_stay(title_element)
            .tag_stay(updated_element)
            .tag_stay(link_element);
        assert_eq!(element, expected_element);
    }

    #[test]
    fn to_xml_with_several_links() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    ..Default::default()
                },
                Link {
                    href: "http://example.net/".to_string(),
                    ..Default::default()
                },
                Link {
                    href: "http://example.org/".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><link href='http://example.com/'/><link href='http://example.net/'/><link href='http://example.org/'/></entry>");
    }

    #[test]
    fn to_xml_with_one_category() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><category term='announcements'/></entry>");
    }

    #[test]
    fn to_xml_with_one_full_category() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    scheme: Some("http://scheme.example/categorization".to_string()),
                    label: Some("Announcements".to_string()),
                },
            ],
            ..Default::default()
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = entry.to_xml();
        let mut expected_element = Element::new(
            "entry".to_string(),
            Some(NS.to_string()),
            vec![]);
        let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
        id_element.text("http://example.com/1".to_string());
        let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
        title_element.text("First!".to_string());
        let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
        updated_element.text("2016-09-17T19:18:32Z".to_string());
        let category_element = Element::new(
            "category".to_string(),
            Some(NS.to_string()),
            vec![
                ("term".to_string(), None, "announcements".to_string()),
                ("scheme".to_string(), None, "http://scheme.example/categorization".to_string()),
                ("label".to_string(), None, "Announcements".to_string()),
            ]);
        expected_element
            .tag_stay(id_element)
            .tag_stay(title_element)
            .tag_stay(updated_element)
            .tag_stay(category_element);
        assert_eq!(element, expected_element);
    }

    #[test]
    fn to_xml_with_several_categories() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    ..Default::default()
                },
                Category {
                    term: "news".to_string(),
                    ..Default::default()
                },
                Category {
                    term: "releases".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><category term='announcements'/><category term='news'/><category term='releases'/></entry>");
    }

    #[test]
    fn to_xml_with_one_author() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><author><name>John Doe</name></author></entry>");
    }

    #[test]
    fn to_xml_with_one_full_author() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    uri: Some("http://john.doe.example/".to_string()),
                    email: Some("john@john.doe.example".to_string()),
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><author><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></author></entry>");
    }

    #[test]
    fn to_xml_with_several_authors() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Mary Smith".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Mustapha Ibrahim".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><author><name>John Doe</name></author><author><name>Mary Smith</name></author><author><name>Mustapha Ibrahim</name></author></entry>");
    }

    #[test]
    fn to_xml_with_one_contributor() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><contributor><name>Corey Farwell</name></contributor></entry>");
    }

    #[test]
    fn to_xml_with_one_full_contributor() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    uri: Some("https://rwell.org/".to_string()),
                    email: Some("coreyf@rwell.org".to_string()),
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><contributor><name>Corey Farwell</name><uri>https://rwell.org/</uri><email>coreyf@rwell.org</email></contributor></entry>");
    }

    #[test]
    fn to_xml_with_several_contributors() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Duncan".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Francis Gagné".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><contributor><name>Corey Farwell</name></contributor><contributor><name>Duncan</name></contributor><contributor><name>Francis Gagné</name></contributor></entry>");
    }

    #[test]
    fn to_xml_with_summary() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            summary: Some("Summary of the first post.".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><summary>Summary of the first post.</summary></entry>");
    }

    #[test]
    fn to_xml_with_content() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            content: Some(Content::Text("Content of the first post.".to_string())),
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><content type='text'>Content of the first post.</content></entry>");
    }

    #[test]
    fn to_xml_with_html_content() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            content: Some(Content::Html("<p>Content of the first post.</p>".to_string())),
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><content type='html'>&lt;p&gt;Content of the first post.&lt;/p&gt;</content></entry>");
    }

    #[test]
    fn to_xml_with_xhtml_content() {
        let mut div = Element::new("div".to_string(), Some(XHTML_NS.to_string()), vec![]);
        let mut p = Element::new("p".to_string(), Some(XHTML_NS.to_string()), vec![]);
        p.text("Content of the first post.".to_string());
        div.children.push(Xml::ElementNode(p));

        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            content: Some(Content::Xhtml(div)),
            ..Default::default()
        };

        let xml = format!("{}", entry.to_xml());
        assert_eq!(xml, "<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><content type='xhtml'><div xmlns='http://www.w3.org/1999/xhtml'><p>Content of the first post.</p></div></content></entry>");
    }

    #[test]
    fn to_xml_with_everything() {
        let entry = Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            published: Some("2016-09-17T19:16:03Z".to_string()),
            source: Some(Source {
                id: Some("http://example.com/feed.atom".to_string()),
                title: Some("Example Domain Updates".to_string()),
                updated: Some("2016-08-07T14:13:49Z".to_string()),
                icon: Some("http://example.com/icon.png".to_string()),
                logo: Some("http://example.com/logo.png".to_string()),
                rights: Some("All rights reversed.".to_string()),
                subtitle: Some("An example in the domain of Internet domains".to_string()),
                generator: Some(Generator {
                    name: "Atom Feed Generator Deluxe".to_string(),
                    uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
                    version: Some("2.0".to_string()),
                }),
                links: vec![
                    Link {
                        href: "http://example.com/".to_string(),
                        rel: Some("alternate".to_string()),
                        mediatype: Some("text/html".to_string()),
                        hreflang: Some("en-US".to_string()),
                        title: Some("Example Domain".to_string()),
                        length: Some("606".to_string()),
                    },
                    Link {
                        href: "http://example.net/".to_string(),
                        ..Default::default()
                    },
                    Link {
                        href: "http://example.org/".to_string(),
                        ..Default::default()
                    },
                ],
                categories: vec![
                    Category {
                        term: "announcements".to_string(),
                        scheme: Some("http://scheme.example/categorization".to_string()),
                        label: Some("Announcements".to_string()),
                    },
                    Category {
                        term: "news".to_string(),
                        ..Default::default()
                    },
                    Category {
                        term: "releases".to_string(),
                        ..Default::default()
                    },
                ],
                authors: vec![
                    Person {
                        name: "John Doe".to_string(),
                        uri: Some("http://john.doe.example/".to_string()),
                        email: Some("john@john.doe.example".to_string()),
                    },
                    Person {
                        name: "Mary Smith".to_string(),
                        ..Default::default()
                    },
                    Person {
                        name: "Mustapha Ibrahim".to_string(),
                        ..Default::default()
                    },
                ],
                contributors: vec![
                    Person {
                        name: "Corey Farwell".to_string(),
                        uri: Some("https://rwell.org/".to_string()),
                        email: Some("coreyf@rwell.org".to_string()),
                    },
                    Person {
                        name: "Duncan".to_string(),
                        ..Default::default()
                    },
                    Person {
                        name: "Francis Gagné".to_string(),
                        ..Default::default()
                    },
                ],
            }),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    rel: Some("alternate".to_string()),
                    mediatype: Some("text/html".to_string()),
                    hreflang: Some("en-US".to_string()),
                    title: Some("Example Domain".to_string()),
                    length: Some("606".to_string()),
                },
                Link {
                    href: "http://example.net/".to_string(),
                    ..Default::default()
                },
                Link {
                    href: "http://example.org/".to_string(),
                    ..Default::default()
                },
            ],
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    scheme: Some("http://scheme.example/categorization".to_string()),
                    label: Some("Announcements".to_string()),
                },
                Category {
                    term: "news".to_string(),
                    ..Default::default()
                },
                Category {
                    term: "releases".to_string(),
                    ..Default::default()
                },
            ],
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    uri: Some("http://john.doe.example/".to_string()),
                    email: Some("john@john.doe.example".to_string()),
                },
                Person {
                    name: "Mary Smith".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Mustapha Ibrahim".to_string(),
                    ..Default::default()
                },
            ],
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    uri: Some("https://rwell.org/".to_string()),
                    email: Some("coreyf@rwell.org".to_string()),
                },
                Person {
                    name: "Duncan".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Francis Gagné".to_string(),
                    ..Default::default()
                },
            ],
            summary: Some("Summary of the first post.".to_string()),
            content: Some(Content::Text("Content of the first post.".to_string())),
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = entry.to_xml();
        let mut expected_element = Element::new(
            "entry".to_string(),
            Some(NS.to_string()),
            vec![]);
        let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
        id_element.text("http://example.com/1".to_string());
        let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
        title_element.text("First!".to_string());
        let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
        updated_element.text("2016-09-17T19:18:32Z".to_string());
        let mut published_element = Element::new("published".to_string(), Some(NS.to_string()), vec![]);
        published_element.text("2016-09-17T19:16:03Z".to_string());
        let mut source_element = Element::new(
            "source".to_string(),
            Some(NS.to_string()),
            vec![]);
        {
            let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
            id_element.text("http://example.com/feed.atom".to_string());
            let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
            title_element.text("Example Domain Updates".to_string());
            let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
            updated_element.text("2016-08-07T14:13:49Z".to_string());
            let mut icon_element = Element::new("icon".to_string(), Some(NS.to_string()), vec![]);
            icon_element.text("http://example.com/icon.png".to_string());
            let mut logo_element = Element::new("logo".to_string(), Some(NS.to_string()), vec![]);
            logo_element.text("http://example.com/logo.png".to_string());
            let mut rights_element = Element::new("rights".to_string(), Some(NS.to_string()), vec![]);
            rights_element.text("All rights reversed.".to_string());
            let mut subtitle_element = Element::new("subtitle".to_string(), Some(NS.to_string()), vec![]);
            subtitle_element.text("An example in the domain of Internet domains".to_string());;
            let mut generator_element = Element::new(
                "generator".to_string(),
                Some(NS.to_string()),
                vec![
                    ("uri".to_string(), None, "https://atom-feed-generator-deluxe.example/".to_string()),
                    ("version".to_string(), None, "2.0".to_string()),
                ]);
            generator_element.text("Atom Feed Generator Deluxe".to_string());
            let link1_element = Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.com/".to_string()),
                    ("rel".to_string(), None, "alternate".to_string()),
                    ("type".to_string(), None, "text/html".to_string()),
                    ("hreflang".to_string(), None, "en-US".to_string()),
                    ("title".to_string(), None, "Example Domain".to_string()),
                    ("length".to_string(), None, "606".to_string()),
                ]);
            let link2_element = Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.net/".to_string()),
                ]);
            let link3_element = Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.org/".to_string()),
                ]);
            let category1_element = Element::new(
                "category".to_string(),
                Some(NS.to_string()),
                vec![
                    ("term".to_string(), None, "announcements".to_string()),
                    ("scheme".to_string(), None, "http://scheme.example/categorization".to_string()),
                    ("label".to_string(), None, "Announcements".to_string()),
                ]);
            let category2_element = Element::new(
                "category".to_string(),
                Some(NS.to_string()),
                vec![
                    ("term".to_string(), None, "news".to_string()),
                ]);
            let category3_element = Element::new(
                "category".to_string(),
                Some(NS.to_string()),
                vec![
                    ("term".to_string(), None, "releases".to_string()),
                ]);
            let mut author1_element = Element::new(
                "author".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            let mut author1_name_element = Element::new(
                "name".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            author1_name_element.text("John Doe".to_string());
            let mut author1_uri_element = Element::new(
                "uri".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            author1_uri_element.text("http://john.doe.example/".to_string());
            let mut author1_email_element = Element::new(
                "email".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            author1_email_element.text("john@john.doe.example".to_string());
            author1_element
                .tag_stay(author1_name_element)
                .tag_stay(author1_uri_element)
                .tag_stay(author1_email_element);
            let mut author2_element = Element::new(
                "author".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            let mut author2_name_element = Element::new(
                "name".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            author2_name_element.text("Mary Smith".to_string());
            author2_element
                .tag_stay(author2_name_element);
            let mut author3_element = Element::new(
                "author".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            let mut author3_name_element = Element::new(
                "name".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            author3_name_element.text("Mustapha Ibrahim".to_string());
            author3_element
                .tag_stay(author3_name_element);
            let mut contributor1_element = Element::new(
                "contributor".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            let mut contributor1_name_element = Element::new(
                "name".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            contributor1_name_element.text("Corey Farwell".to_string());
            let mut contributor1_uri_element = Element::new(
                "uri".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            contributor1_uri_element.text("https://rwell.org/".to_string());
            let mut contributor1_email_element = Element::new(
                "email".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            contributor1_email_element.text("coreyf@rwell.org".to_string());
            contributor1_element
                .tag_stay(contributor1_name_element)
                .tag_stay(contributor1_uri_element)
                .tag_stay(contributor1_email_element);
            let mut contributor2_element = Element::new(
                "contributor".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            let mut contributor2_name_element = Element::new(
                "name".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            contributor2_name_element.text("Duncan".to_string());
            contributor2_element
                .tag_stay(contributor2_name_element);
            let mut contributor3_element = Element::new(
                "contributor".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            let mut contributor3_name_element = Element::new(
                "name".to_string(),
                Some(NS.to_string()),
                vec![],
            );
            contributor3_name_element.text("Francis Gagné".to_string());
            contributor3_element
                .tag_stay(contributor3_name_element);
            source_element
                .tag_stay(id_element)
                .tag_stay(title_element)
                .tag_stay(updated_element)
                .tag_stay(icon_element)
                .tag_stay(logo_element)
                .tag_stay(rights_element)
                .tag_stay(subtitle_element)
                .tag_stay(generator_element)
                .tag_stay(link1_element)
                .tag_stay(link2_element)
                .tag_stay(link3_element)
                .tag_stay(category1_element)
                .tag_stay(category2_element)
                .tag_stay(category3_element)
                .tag_stay(author1_element)
                .tag_stay(author2_element)
                .tag_stay(author3_element)
                .tag_stay(contributor1_element)
                .tag_stay(contributor2_element)
                .tag_stay(contributor3_element);
        };
        let link1_element = Element::new(
            "link".to_string(),
            Some(NS.to_string()),
            vec![
                ("href".to_string(), None, "http://example.com/".to_string()),
                ("rel".to_string(), None, "alternate".to_string()),
                ("type".to_string(), None, "text/html".to_string()),
                ("hreflang".to_string(), None, "en-US".to_string()),
                ("title".to_string(), None, "Example Domain".to_string()),
                ("length".to_string(), None, "606".to_string()),
            ]);
        let link2_element = Element::new(
            "link".to_string(),
            Some(NS.to_string()),
            vec![
                ("href".to_string(), None, "http://example.net/".to_string()),
            ]);
        let link3_element = Element::new(
            "link".to_string(),
            Some(NS.to_string()),
            vec![
                ("href".to_string(), None, "http://example.org/".to_string()),
            ]);
        let category1_element = Element::new(
            "category".to_string(),
            Some(NS.to_string()),
            vec![
                ("term".to_string(), None, "announcements".to_string()),
                ("scheme".to_string(), None, "http://scheme.example/categorization".to_string()),
                ("label".to_string(), None, "Announcements".to_string()),
            ]);
        let category2_element = Element::new(
            "category".to_string(),
            Some(NS.to_string()),
            vec![
                ("term".to_string(), None, "news".to_string()),
            ]);
        let category3_element = Element::new(
            "category".to_string(),
            Some(NS.to_string()),
            vec![
                ("term".to_string(), None, "releases".to_string()),
            ]);
        let mut author1_element = Element::new(
            "author".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        let mut author1_name_element = Element::new(
            "name".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        author1_name_element.text("John Doe".to_string());
        let mut author1_uri_element = Element::new(
            "uri".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        author1_uri_element.text("http://john.doe.example/".to_string());
        let mut author1_email_element = Element::new(
            "email".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        author1_email_element.text("john@john.doe.example".to_string());
        author1_element
            .tag_stay(author1_name_element)
            .tag_stay(author1_uri_element)
            .tag_stay(author1_email_element);
        let mut author2_element = Element::new(
            "author".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        let mut author2_name_element = Element::new(
            "name".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        author2_name_element.text("Mary Smith".to_string());
        author2_element
            .tag_stay(author2_name_element);
        let mut author3_element = Element::new(
            "author".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        let mut author3_name_element = Element::new(
            "name".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        author3_name_element.text("Mustapha Ibrahim".to_string());
        author3_element
            .tag_stay(author3_name_element);
        let mut contributor1_element = Element::new(
            "contributor".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        let mut contributor1_name_element = Element::new(
            "name".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        contributor1_name_element.text("Corey Farwell".to_string());
        let mut contributor1_uri_element = Element::new(
            "uri".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        contributor1_uri_element.text("https://rwell.org/".to_string());
        let mut contributor1_email_element = Element::new(
            "email".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        contributor1_email_element.text("coreyf@rwell.org".to_string());
        contributor1_element
            .tag_stay(contributor1_name_element)
            .tag_stay(contributor1_uri_element)
            .tag_stay(contributor1_email_element);
        let mut contributor2_element = Element::new(
            "contributor".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        let mut contributor2_name_element = Element::new(
            "name".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        contributor2_name_element.text("Duncan".to_string());
        contributor2_element
            .tag_stay(contributor2_name_element);
        let mut contributor3_element = Element::new(
            "contributor".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        let mut contributor3_name_element = Element::new(
            "name".to_string(),
            Some(NS.to_string()),
            vec![],
        );
        contributor3_name_element.text("Francis Gagné".to_string());
        contributor3_element
            .tag_stay(contributor3_name_element);
        let mut summary_element = Element::new("summary".to_string(), Some(NS.to_string()), vec![]);
        summary_element.text("Summary of the first post.".to_string());
        let mut content_element = Element::new(
            "content".to_string(),
            Some(NS.to_string()),
            vec![("type".to_string(), None, "text".to_string())]
        );
        content_element.text("Content of the first post.".to_string());
        expected_element
            .tag_stay(id_element)
            .tag_stay(title_element)
            .tag_stay(updated_element)
            .tag_stay(published_element)
            .tag_stay(source_element)
            .tag_stay(link1_element)
            .tag_stay(link2_element)
            .tag_stay(link3_element)
            .tag_stay(category1_element)
            .tag_stay(category2_element)
            .tag_stay(category3_element)
            .tag_stay(author1_element)
            .tag_stay(author2_element)
            .tag_stay(author3_element)
            .tag_stay(contributor1_element)
            .tag_stay(contributor2_element)
            .tag_stay(contributor3_element)
            .tag_stay(summary_element)
            .tag_stay(content_element);
        assert_eq!(element, expected_element);
    }

    #[test]
    fn from_xml_empty() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'/>").unwrap());
        assert!(entry.is_err());
    }

    #[test]
    fn from_xml_missing_id() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><title>First!</title><updated>2016-09-17T19:18:32Z</updated></entry>").unwrap());
        assert_eq!(entry, Err("<entry> is missing required <id> element"));
    }

    #[test]
    fn from_xml_missing_title() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><updated>2016-09-17T19:18:32Z</updated></entry>").unwrap());
        assert_eq!(entry, Err("<entry> is missing required <title> element"));
    }

    #[test]
    fn from_xml_missing_updated() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title></entry>").unwrap());
        assert_eq!(entry, Err("<entry> is missing required <updated> element"));
    }

    #[test]
    fn from_xml_minimal() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_published() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><published>2016-09-17T19:16:03Z</published></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            published: Some("2016-09-17T19:16:03Z".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_source() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><source/></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            source: Some(Source::default()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_invalid_source() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><source><generator/></source></entry>").unwrap());
        assert_eq!(entry, Err("<generator> is missing required name"));
    }

    #[test]
    fn from_xml_with_full_source() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'>
            <id>http://example.com/1</id>
            <title>First!</title>
            <updated>2016-09-17T19:18:32Z</updated>
            <source>
                <id>http://example.com/feed.atom</id>
                <title>Example Domain Updates</title>
                <updated>2016-08-07T14:13:49Z</updated>
                <icon>http://example.com/icon.png</icon>
                <logo>http://example.com/logo.png</logo>
                <rights>All rights reversed.</rights>
                <subtitle>An example in the domain of Internet domains</subtitle>
                <generator>Atom Feed Generator Deluxe</generator>
                <link href='http://example.com/'/>
                <category term='announcements'/>
                <author>
                    <name>John Doe</name>
                </author>
                <contributor>
                    <name>Corey Farwell</name>
                </contributor>
            </source>
        </entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            source: Some(Source {
                id: Some("http://example.com/feed.atom".to_string()),
                title: Some("Example Domain Updates".to_string()),
                updated: Some("2016-08-07T14:13:49Z".to_string()),
                icon: Some("http://example.com/icon.png".to_string()),
                logo: Some("http://example.com/logo.png".to_string()),
                rights: Some("All rights reversed.".to_string()),
                subtitle: Some("An example in the domain of Internet domains".to_string()),
                generator: Some(Generator {
                    name: "Atom Feed Generator Deluxe".to_string(),
                    ..Default::default()
                }),
                links: vec![
                    Link {
                        href: "http://example.com/".to_string(),
                        ..Default::default()
                    },
                ],
                categories: vec![
                    Category {
                        term: "announcements".to_string(),
                        ..Default::default()
                    },
                ],
                authors: vec![
                    Person {
                        name: "John Doe".to_string(),
                        ..Default::default()
                    },
                ],
                contributors: vec![
                    Person {
                        name: "Corey Farwell".to_string(),
                        ..Default::default()
                    },
                ],
            }),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_invalid_link() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><link/></entry>").unwrap());
        assert_eq!(entry, Err(r#"<link> is missing required "href" attribute"#));
    }

    #[test]
    fn from_xml_with_one_link() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><link href='http://example.com/'/></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_one_full_link() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><link href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    rel: Some("alternate".to_string()),
                    mediatype: Some("text/html".to_string()),
                    hreflang: Some("en-US".to_string()),
                    title: Some("Example Domain".to_string()),
                    length: Some("606".to_string()),
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_several_links() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><link href='http://example.com/'/><link href='http://example.net/'/><link href='http://example.org/'/></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    ..Default::default()
                },
                Link {
                    href: "http://example.net/".to_string(),
                    ..Default::default()
                },
                Link {
                    href: "http://example.org/".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_invalid_category() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><category/></entry>").unwrap());
        assert_eq!(entry, Err(r#"<category> is missing required "term" attribute"#));
    }

    #[test]
    fn from_xml_with_one_category() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><category term='announcements'/></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_one_full_category() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><category term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    scheme: Some("http://scheme.example/categorization".to_string()),
                    label: Some("Announcements".to_string()),
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_several_categories() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><category term='announcements'/><category term='news'/><category term='releases'/></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    ..Default::default()
                },
                Category {
                    term: "news".to_string(),
                    ..Default::default()
                },
                Category {
                    term: "releases".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_invalid_author() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><author/></entry>").unwrap());
        assert_eq!(entry, Err("<author> is missing required <name> element"));
    }

    #[test]
    fn from_xml_with_one_author() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><author><name>John Doe</name></author></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_one_full_author() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><author><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></author></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    uri: Some("http://john.doe.example/".to_string()),
                    email: Some("john@john.doe.example".to_string()),
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_several_authors() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><author><name>John Doe</name></author><author><name>Mary Smith</name></author><author><name>Mustapha Ibrahim</name></author></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Mary Smith".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Mustapha Ibrahim".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_invalid_contributor() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><contributor/></entry>").unwrap());
        assert_eq!(entry, Err("<contributor> is missing required <name> element"));
    }

    #[test]
    fn from_xml_with_one_contributor() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><contributor><name>Corey Farwell</name></contributor></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_one_full_contributor() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><contributor><name>Corey Farwell</name><uri>https://rwell.org/</uri><email>coreyf@rwell.org</email></contributor></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    uri: Some("https://rwell.org/".to_string()),
                    email: Some("coreyf@rwell.org".to_string()),
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_several_contributors() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><contributor><name>Corey Farwell</name></contributor><contributor><name>Duncan</name></contributor><contributor><name>Francis Gagné</name></contributor></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Duncan".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Francis Gagné".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_summary() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><summary>Summary of the first post.</summary></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            summary: Some("Summary of the first post.".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_content() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><content>Content of the first post.</content></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            content: Some(Content::Text("Content of the first post.".to_string())),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_html_content() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><content type='html'>&lt;p&gt;Content of the first post.&lt;/p&gt;</content></entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            content: Some(Content::Html("<p>Content of the first post.</p>".to_string())),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_xhtml_content() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><content type='xhtml'><div xmlns='http://www.w3.org/1999/xhtml'><p>Content of the first post.</p></div></content></entry>").unwrap());
        let namespace_attr = ("xmlns".to_string(), None, "http://www.w3.org/1999/xhtml".to_string());
        let mut div = Element::new("div".to_string(), Some(XHTML_NS.to_string()), vec![namespace_attr]);
        let mut p = Element::new("p".to_string(), Some(XHTML_NS.to_string()), vec![]);
        p.text("Content of the first post.".to_string());
        div.children.push(Xml::ElementNode(p));

        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            content: Some(Content::Xhtml(div)),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_xhtml_content_no_div() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><content type='xhtml'><p>Content of the first post.</p></content></entry>").unwrap());
        assert_eq!(entry, Err("expected to find child element <div> of <content> but found none"));
    }

    #[test]
    fn from_xml_with_invalid_content_type() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/1</id><title>First!</title><updated>2016-09-17T19:18:32Z</updated><content type='invalid'>Content of the first post.</content></entry>").unwrap());
        assert_eq!(entry, Err("<content> has unknown type"));
    }

    #[test]
    fn from_xml_with_everything() {
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'>
            <id>http://example.com/1</id>
            <title>First!</title>
            <updated>2016-09-17T19:18:32Z</updated>
            <published>2016-09-17T19:16:03Z</published>
            <source>
                <id>http://example.com/feed.atom</id>
                <title>Example Domain Updates</title>
                <updated>2016-08-07T14:13:49Z</updated>
                <icon>http://example.com/icon.png</icon>
                <logo>http://example.com/logo.png</logo>
                <rights>All rights reversed.</rights>
                <subtitle>An example in the domain of Internet domains</subtitle>
                <generator uri='https://atom-feed-generator-deluxe.example/' version='2.0'>Atom Feed Generator Deluxe</generator>
                <link href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/>
                <link href='http://example.net/'/>
                <link href='http://example.org/'/>
                <category term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/>
                <category term='news'/>
                <category term='releases'/>
                <author>
                    <name>John Doe</name>
                    <uri>http://john.doe.example/</uri>
                    <email>john@john.doe.example</email>
                </author>
                <author>
                    <name>Mary Smith</name>
                </author>
                <author>
                    <name>Mustapha Ibrahim</name>
                </author>
                <contributor>
                    <name>Corey Farwell</name>
                    <uri>https://rwell.org/</uri>
                    <email>coreyf@rwell.org</email>
                </contributor>
                <contributor>
                    <name>Duncan</name>
                </contributor>
                <contributor>
                    <name>Francis Gagné</name>
                </contributor>
            </source>
            <link href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/>
            <link href='http://example.net/'/>
            <link href='http://example.org/'/>
            <category term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/>
            <category term='news'/>
            <category term='releases'/>
            <author>
                <name>John Doe</name>
                <uri>http://john.doe.example/</uri>
                <email>john@john.doe.example</email>
            </author>
            <author>
                <name>Mary Smith</name>
            </author>
            <author>
                <name>Mustapha Ibrahim</name>
            </author>
            <contributor>
                <name>Corey Farwell</name>
                <uri>https://rwell.org/</uri>
                <email>coreyf@rwell.org</email>
            </contributor>
            <contributor>
                <name>Duncan</name>
            </contributor>
            <contributor>
                <name>Francis Gagné</name>
            </contributor>
            <summary>Summary of the first post.</summary>
            <content>Content of the first post.</content>
        </entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            published: Some("2016-09-17T19:16:03Z".to_string()),
            source: Some(Source {
                id: Some("http://example.com/feed.atom".to_string()),
                title: Some("Example Domain Updates".to_string()),
                updated: Some("2016-08-07T14:13:49Z".to_string()),
                icon: Some("http://example.com/icon.png".to_string()),
                logo: Some("http://example.com/logo.png".to_string()),
                rights: Some("All rights reversed.".to_string()),
                subtitle: Some("An example in the domain of Internet domains".to_string()),
                generator: Some(Generator {
                    name: "Atom Feed Generator Deluxe".to_string(),
                    uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
                    version: Some("2.0".to_string()),
                }),
                links: vec![
                    Link {
                        href: "http://example.com/".to_string(),
                        rel: Some("alternate".to_string()),
                        mediatype: Some("text/html".to_string()),
                        hreflang: Some("en-US".to_string()),
                        title: Some("Example Domain".to_string()),
                        length: Some("606".to_string()),
                    },
                    Link {
                        href: "http://example.net/".to_string(),
                        ..Default::default()
                    },
                    Link {
                        href: "http://example.org/".to_string(),
                        ..Default::default()
                    },
                ],
                categories: vec![
                    Category {
                        term: "announcements".to_string(),
                        scheme: Some("http://scheme.example/categorization".to_string()),
                        label: Some("Announcements".to_string()),
                    },
                    Category {
                        term: "news".to_string(),
                        ..Default::default()
                    },
                    Category {
                        term: "releases".to_string(),
                        ..Default::default()
                    },
                ],
                authors: vec![
                    Person {
                        name: "John Doe".to_string(),
                        uri: Some("http://john.doe.example/".to_string()),
                        email: Some("john@john.doe.example".to_string()),
                    },
                    Person {
                        name: "Mary Smith".to_string(),
                        ..Default::default()
                    },
                    Person {
                        name: "Mustapha Ibrahim".to_string(),
                        ..Default::default()
                    },
                ],
                contributors: vec![
                    Person {
                        name: "Corey Farwell".to_string(),
                        uri: Some("https://rwell.org/".to_string()),
                        email: Some("coreyf@rwell.org".to_string()),
                    },
                    Person {
                        name: "Duncan".to_string(),
                        ..Default::default()
                    },
                    Person {
                        name: "Francis Gagné".to_string(),
                        ..Default::default()
                    },
                ],
            }),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    rel: Some("alternate".to_string()),
                    mediatype: Some("text/html".to_string()),
                    hreflang: Some("en-US".to_string()),
                    title: Some("Example Domain".to_string()),
                    length: Some("606".to_string()),
                },
                Link {
                    href: "http://example.net/".to_string(),
                    ..Default::default()
                },
                Link {
                    href: "http://example.org/".to_string(),
                    ..Default::default()
                },
            ],
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    scheme: Some("http://scheme.example/categorization".to_string()),
                    label: Some("Announcements".to_string()),
                },
                Category {
                    term: "news".to_string(),
                    ..Default::default()
                },
                Category {
                    term: "releases".to_string(),
                    ..Default::default()
                },
            ],
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    uri: Some("http://john.doe.example/".to_string()),
                    email: Some("john@john.doe.example".to_string()),
                },
                Person {
                    name: "Mary Smith".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Mustapha Ibrahim".to_string(),
                    ..Default::default()
                },
            ],
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    uri: Some("https://rwell.org/".to_string()),
                    email: Some("coreyf@rwell.org".to_string()),
                },
                Person {
                    name: "Duncan".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Francis Gagné".to_string(),
                    ..Default::default()
                },
            ],
            summary: Some("Summary of the first post.".to_string()),
            content: Some(Content::Text("Content of the first post.".to_string())),
        }));
    }

    #[test]
    fn from_xml_with_everything_shuffled() {
        // $ The "atom:entry" Element: "This specification assigns no significance to the order of appearance of the child elements of atom:entry."
        let entry = Entry::from_xml(&str::parse("<entry xmlns='http://www.w3.org/2005/Atom'>
            <link href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/>
            <link href='http://example.net/'/>
            <content>Content of the first post.</content>
            <source>
                <id>http://example.com/feed.atom</id>
                <title>Example Domain Updates</title>
                <updated>2016-08-07T14:13:49Z</updated>
                <icon>http://example.com/icon.png</icon>
                <logo>http://example.com/logo.png</logo>
                <rights>All rights reversed.</rights>
                <subtitle>An example in the domain of Internet domains</subtitle>
                <generator uri='https://atom-feed-generator-deluxe.example/' version='2.0'>Atom Feed Generator Deluxe</generator>
                <link href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/>
                <link href='http://example.net/'/>
                <link href='http://example.org/'/>
                <category term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/>
                <category term='news'/>
                <category term='releases'/>
                <author>
                    <name>John Doe</name>
                    <uri>http://john.doe.example/</uri>
                    <email>john@john.doe.example</email>
                </author>
                <author>
                    <name>Mary Smith</name>
                </author>
                <author>
                    <name>Mustapha Ibrahim</name>
                </author>
                <contributor>
                    <name>Corey Farwell</name>
                    <uri>https://rwell.org/</uri>
                    <email>coreyf@rwell.org</email>
                </contributor>
                <contributor>
                    <name>Duncan</name>
                </contributor>
                <contributor>
                    <name>Francis Gagné</name>
                </contributor>
            </source>
            <title>First!</title>
            <contributor>
                <name>Corey Farwell</name>
                <uri>https://rwell.org/</uri>
                <email>coreyf@rwell.org</email>
            </contributor>
            <updated>2016-09-17T19:18:32Z</updated>
            <id>http://example.com/1</id>
            <category term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/>
            <contributor>
                <name>Duncan</name>
            </contributor>
            <published>2016-09-17T19:16:03Z</published>
            <author>
                <name>John Doe</name>
                <uri>http://john.doe.example/</uri>
                <email>john@john.doe.example</email>
            </author>
            <category term='news'/>
            <summary>Summary of the first post.</summary>
            <author>
                <name>Mary Smith</name>
            </author>
            <contributor>
                <name>Francis Gagné</name>
            </contributor>
            <link href='http://example.org/'/>
            <author>
                <name>Mustapha Ibrahim</name>
            </author>
            <category term='releases'/>
        </entry>").unwrap());
        assert_eq!(entry, Ok(Entry {
            id: "http://example.com/1".to_string(),
            title: "First!".to_string(),
            updated: "2016-09-17T19:18:32Z".to_string(),
            published: Some("2016-09-17T19:16:03Z".to_string()),
            source: Some(Source {
                id: Some("http://example.com/feed.atom".to_string()),
                title: Some("Example Domain Updates".to_string()),
                updated: Some("2016-08-07T14:13:49Z".to_string()),
                icon: Some("http://example.com/icon.png".to_string()),
                logo: Some("http://example.com/logo.png".to_string()),
                rights: Some("All rights reversed.".to_string()),
                subtitle: Some("An example in the domain of Internet domains".to_string()),
                generator: Some(Generator {
                    name: "Atom Feed Generator Deluxe".to_string(),
                    uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
                    version: Some("2.0".to_string()),
                }),
                links: vec![
                    Link {
                        href: "http://example.com/".to_string(),
                        rel: Some("alternate".to_string()),
                        mediatype: Some("text/html".to_string()),
                        hreflang: Some("en-US".to_string()),
                        title: Some("Example Domain".to_string()),
                        length: Some("606".to_string()),
                    },
                    Link {
                        href: "http://example.net/".to_string(),
                        ..Default::default()
                    },
                    Link {
                        href: "http://example.org/".to_string(),
                        ..Default::default()
                    },
                ],
                categories: vec![
                    Category {
                        term: "announcements".to_string(),
                        scheme: Some("http://scheme.example/categorization".to_string()),
                        label: Some("Announcements".to_string()),
                    },
                    Category {
                        term: "news".to_string(),
                        ..Default::default()
                    },
                    Category {
                        term: "releases".to_string(),
                        ..Default::default()
                    },
                ],
                authors: vec![
                    Person {
                        name: "John Doe".to_string(),
                        uri: Some("http://john.doe.example/".to_string()),
                        email: Some("john@john.doe.example".to_string()),
                    },
                    Person {
                        name: "Mary Smith".to_string(),
                        ..Default::default()
                    },
                    Person {
                        name: "Mustapha Ibrahim".to_string(),
                        ..Default::default()
                    },
                ],
                contributors: vec![
                    Person {
                        name: "Corey Farwell".to_string(),
                        uri: Some("https://rwell.org/".to_string()),
                        email: Some("coreyf@rwell.org".to_string()),
                    },
                    Person {
                        name: "Duncan".to_string(),
                        ..Default::default()
                    },
                    Person {
                        name: "Francis Gagné".to_string(),
                        ..Default::default()
                    },
                ],
            }),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    rel: Some("alternate".to_string()),
                    mediatype: Some("text/html".to_string()),
                    hreflang: Some("en-US".to_string()),
                    title: Some("Example Domain".to_string()),
                    length: Some("606".to_string()),
                },
                Link {
                    href: "http://example.net/".to_string(),
                    ..Default::default()
                },
                Link {
                    href: "http://example.org/".to_string(),
                    ..Default::default()
                },
            ],
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    scheme: Some("http://scheme.example/categorization".to_string()),
                    label: Some("Announcements".to_string()),
                },
                Category {
                    term: "news".to_string(),
                    ..Default::default()
                },
                Category {
                    term: "releases".to_string(),
                    ..Default::default()
                },
            ],
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    uri: Some("http://john.doe.example/".to_string()),
                    email: Some("john@john.doe.example".to_string()),
                },
                Person {
                    name: "Mary Smith".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Mustapha Ibrahim".to_string(),
                    ..Default::default()
                },
            ],
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    uri: Some("https://rwell.org/".to_string()),
                    email: Some("coreyf@rwell.org".to_string()),
                },
                Person {
                    name: "Duncan".to_string(),
                    ..Default::default()
                },
                Person {
                    name: "Francis Gagné".to_string(),
                    ..Default::default()
                },
            ],
            summary: Some("Summary of the first post.".to_string()),
            content: Some(Content::Text("Content of the first post.".to_string())),
        }));
    }
}
