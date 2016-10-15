use std::str::FromStr;
use xml::{Element, ElementBuilder, Parser, Xml};

use {Category, Entry, Generator, Link, NS, Person};
use author::Author;
use contributor::Contributor;
use utils::{ElementUtils, Flip, FromXml, ToXml};


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
#[derive(Clone, Default, Debug, PartialEq)]
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


impl ToXml for Feed {
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
            feed.tag(Author(person).to_xml());
        }

        for person in &self.contributors {
            feed.tag(Contributor(person).to_xml());
        }

        for entry in &self.entries {
            feed.tag(entry.to_xml());
        }

        feed
    }
}


impl FromXml for Feed {
    fn from_xml(elem: &Element) -> Result<Self, &'static str> {
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

        let entries = try!(elem.get_children("entry", Some(NS))
            .map(|e| FromXml::from_xml(e))
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
                return FromXml::from_xml(&elem);
            }
        }

        Err("Atom read error")
    }
}


impl ToString for Feed {
    fn to_string(&self) -> String {
        format!("{}{}", Xml::PINode(r#"xml version="1.0" encoding="utf-8""#.to_string()), self.to_xml())
    }
}


#[cfg(test)]
mod tests {
    use std::str::{self, FromStr};

    use xml::Element;

    use {Content, Category, Entry, Feed, Generator, Link, NS, Person, Source};
    use utils::{FromXml, ToXml};

    #[test]
    fn to_xml_minimal() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated></feed>");
    }

    #[test]
    fn to_xml_with_icon() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            icon: Some("http://example.com/icon.png".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><icon>http://example.com/icon.png</icon></feed>");
    }

    #[test]
    fn to_xml_with_logo() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            logo: Some("http://example.com/logo.png".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><logo>http://example.com/logo.png</logo></feed>");
    }

    #[test]
    fn to_xml_with_rights() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            rights: Some("All rights reversed.".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><rights>All rights reversed.</rights></feed>");
    }

    #[test]
    fn to_xml_with_subtitle() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            subtitle: Some("An example in the domain of Internet domains".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><subtitle>An example in the domain of Internet domains</subtitle></feed>");
    }

    #[test]
    fn to_xml_with_generator() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            generator: Some(Generator {
                name: "Atom Feed Generator Deluxe".to_string(),
                ..Generator::default()
            }),
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><generator>Atom Feed Generator Deluxe</generator></feed>");
    }

    #[test]
    fn to_xml_with_full_generator() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            generator: Some(Generator {
                name: "Atom Feed Generator Deluxe".to_string(),
                uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
                version: Some("2.0".to_string()),
            }),
            ..Default::default()
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = feed.to_xml();
        let mut expected_element = Element::new(
            "feed".to_string(),
            Some(NS.to_string()),
            vec![]);
        let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
        id_element.text("http://example.com/feed.atom".to_string());
        let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
        title_element.text("Examplar Feed".to_string());
        let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
        updated_element.text("2016-09-18T18:53:16Z".to_string());
        let mut generator_element = Element::new(
            "generator".to_string(),
            Some(NS.to_string()),
            vec![
                ("uri".to_string(), None, "https://atom-feed-generator-deluxe.example/".to_string()),
                ("version".to_string(), None, "2.0".to_string()),
            ]);
        generator_element.text("Atom Feed Generator Deluxe".to_string());
        expected_element
            .tag_stay(id_element)
            .tag_stay(title_element)
            .tag_stay(updated_element)
            .tag_stay(generator_element);
        assert_eq!(element, expected_element);
    }

    #[test]
    fn to_xml_with_one_link() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><link href='http://example.com/'/></feed>");
    }

    #[test]
    fn to_xml_with_one_full_link() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let element = feed.to_xml();
        let mut expected_element = Element::new(
            "feed".to_string(),
            Some(NS.to_string()),
            vec![]);
        let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
        id_element.text("http://example.com/feed.atom".to_string());
        let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
        title_element.text("Examplar Feed".to_string());
        let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
        updated_element.text("2016-09-18T18:53:16Z".to_string());
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
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><link href='http://example.com/'/><link href='http://example.net/'/><link href='http://example.org/'/></feed>");
    }

    #[test]
    fn to_xml_with_one_category() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><category term='announcements'/></feed>");
    }

    #[test]
    fn to_xml_with_one_full_category() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let element = feed.to_xml();
        let mut expected_element = Element::new(
            "feed".to_string(),
            Some(NS.to_string()),
            vec![]);
        let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
        id_element.text("http://example.com/feed.atom".to_string());
        let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
        title_element.text("Examplar Feed".to_string());
        let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
        updated_element.text("2016-09-18T18:53:16Z".to_string());
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
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><category term='announcements'/><category term='news'/><category term='releases'/></feed>");
    }

    #[test]
    fn to_xml_with_one_author() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><author><name>John Doe</name></author></feed>");
    }

    #[test]
    fn to_xml_with_one_full_author() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    uri: Some("http://john.doe.example/".to_string()),
                    email: Some("john@john.doe.example".to_string()),
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><author><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></author></feed>");
    }

    #[test]
    fn to_xml_with_several_authors() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><author><name>John Doe</name></author><author><name>Mary Smith</name></author><author><name>Mustapha Ibrahim</name></author></feed>");
    }

    #[test]
    fn to_xml_with_one_contributor() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><contributor><name>Corey Farwell</name></contributor></feed>");
    }

    #[test]
    fn to_xml_with_one_full_contributor() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    uri: Some("https://rwell.org/".to_string()),
                    email: Some("coreyf@rwell.org".to_string()),
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><contributor><name>Corey Farwell</name><uri>https://rwell.org/</uri><email>coreyf@rwell.org</email></contributor></feed>");
    }

    #[test]
    fn to_xml_with_several_contributors() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><contributor><name>Corey Farwell</name></contributor><contributor><name>Duncan</name></contributor><contributor><name>Francis Gagné</name></contributor></feed>");
    }

    #[test]
    fn to_xml_with_one_entry() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            entries: vec![
                Entry {
                    id: "http://example.com/1".to_string(),
                    title: "First!".to_string(),
                    updated: "2016-09-17T19:18:32Z".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'>\
            <id>http://example.com/feed.atom</id>\
            <title>Examplar Feed</title>\
            <updated>2016-09-18T18:53:16Z</updated>\
            <entry>\
                <id>http://example.com/1</id>\
                <title>First!</title>\
                <updated>2016-09-17T19:18:32Z</updated>\
            </entry>\
        </feed>");
    }

    #[test]
    fn to_xml_with_one_full_entry() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            entries: vec![
                Entry {
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
                    summary: Some("Summary of the first post.".to_string()),
                    content: Some(Content::Text("Content of the first post.".to_string())),
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'>\
            <id>http://example.com/feed.atom</id>\
            <title>Examplar Feed</title>\
            <updated>2016-09-18T18:53:16Z</updated>\
            <entry>\
                <id>http://example.com/1</id>\
                <title>First!</title>\
                <updated>2016-09-17T19:18:32Z</updated>\
                <published>2016-09-17T19:16:03Z</published>\
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
                <link href='http://example.com/'/>\
                <category term='announcements'/>\
                <author>\
                    <name>John Doe</name>\
                </author>\
                <contributor>\
                    <name>Corey Farwell</name>\
                </contributor>\
                <summary>Summary of the first post.</summary>\
                <content type='text'>Content of the first post.</content>\
            </entry>\
        </feed>");
    }

    #[test]
    fn to_xml_with_several_entries() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            entries: vec![
                Entry {
                    id: "http://example.com/3".to_string(),
                    title: "Third!".to_string(),
                    updated: "2016-09-18T18:53:16Z".to_string(),
                    ..Default::default()
                },
                Entry {
                    id: "http://example.com/2".to_string(),
                    title: "Second!".to_string(),
                    updated: "2016-09-18T11:22:55Z".to_string(),
                    ..Default::default()
                },
                Entry {
                    id: "http://example.com/1".to_string(),
                    title: "First!".to_string(),
                    updated: "2016-09-17T19:18:32Z".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", feed.to_xml());
        assert_eq!(xml, "<feed xmlns='http://www.w3.org/2005/Atom'>\
            <id>http://example.com/feed.atom</id>\
            <title>Examplar Feed</title>\
            <updated>2016-09-18T18:53:16Z</updated>\
            <entry>\
                <id>http://example.com/3</id>\
                <title>Third!</title>\
                <updated>2016-09-18T18:53:16Z</updated>\
            </entry>\
            <entry>\
                <id>http://example.com/2</id>\
                <title>Second!</title>\
                <updated>2016-09-18T11:22:55Z</updated>\
            </entry>\
            <entry>\
                <id>http://example.com/1</id>\
                <title>First!</title>\
                <updated>2016-09-17T19:18:32Z</updated>\
            </entry>\
        </feed>");
    }

    #[test]
    fn to_xml_with_everything() {
        let feed = Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Example Domain Updates".to_string(),
            updated: "2016-08-07T14:13:49Z".to_string(),
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
            entries: vec![
                Entry {
                    id: "http://example.com/3".to_string(),
                    title: "Third!".to_string(),
                    updated: "2016-09-18T18:53:16Z".to_string(),
                    published: Some("2016-09-18T18:53:16Z".to_string()),
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
                },
                Entry {
                    id: "http://example.com/2".to_string(),
                    title: "Second!".to_string(),
                    updated: "2016-09-18T11:22:55Z".to_string(),
                    ..Default::default()
                },
                Entry {
                    id: "http://example.com/1".to_string(),
                    title: "First!".to_string(),
                    updated: "2016-09-17T19:18:32Z".to_string(),
                    ..Default::default()
                },
            ],
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = feed.to_xml();
        let mut expected_element = Element::new(
            "feed".to_string(),
            Some(NS.to_string()),
            vec![]);
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
        subtitle_element.text("An example in the domain of Internet domains".to_string());
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
        let mut entry1_element = Element::new(
            "entry".to_string(),
            Some(NS.to_string()),
            vec![]);
        {
            let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
            id_element.text("http://example.com/3".to_string());
            let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
            title_element.text("Third!".to_string());
            let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
            updated_element.text("2016-09-18T18:53:16Z".to_string());
            let mut published_element = Element::new("published".to_string(), Some(NS.to_string()), vec![]);
            published_element.text("2016-09-18T18:53:16Z".to_string());
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
            entry1_element
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
        }
        let mut entry2_element = Element::new(
            "entry".to_string(),
            Some(NS.to_string()),
            vec![]);
        {
            let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
            id_element.text("http://example.com/2".to_string());
            let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
            title_element.text("Second!".to_string());
            let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
            updated_element.text("2016-09-18T11:22:55Z".to_string());
            entry2_element
                .tag_stay(id_element)
                .tag_stay(title_element)
                .tag_stay(updated_element);
        }
        let mut entry3_element = Element::new(
            "entry".to_string(),
            Some(NS.to_string()),
            vec![]);
        {
            let mut id_element = Element::new("id".to_string(), Some(NS.to_string()), vec![]);
            id_element.text("http://example.com/1".to_string());
            let mut title_element = Element::new("title".to_string(), Some(NS.to_string()), vec![]);
            title_element.text("First!".to_string());
            let mut updated_element = Element::new("updated".to_string(), Some(NS.to_string()), vec![]);
            updated_element.text("2016-09-17T19:18:32Z".to_string());
            entry3_element
                .tag_stay(id_element)
                .tag_stay(title_element)
                .tag_stay(updated_element);
        }
        expected_element
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
            .tag_stay(contributor3_element)
            .tag_stay(entry1_element)
            .tag_stay(entry2_element)
            .tag_stay(entry3_element);
        assert_eq!(element, expected_element);
    }

    #[test]
    fn from_xml_empty() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'/>").unwrap());
        assert!(feed.is_err());
    }

    #[test]
    fn from_xml_missing_id() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated></feed>").unwrap());
        assert_eq!(feed, Err("<feed> is missing required <id> element"));
    }

    #[test]
    fn from_xml_missing_title() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><updated>2016-09-18T18:53:16Z</updated></feed>").unwrap());
        assert_eq!(feed, Err("<feed> is missing required <title> element"));
    }

    #[test]
    fn from_xml_missing_updated() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title></feed>").unwrap());
        assert_eq!(feed, Err("<feed> is missing required <updated> element"));
    }

    #[test]
    fn from_xml_minimal() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_icon() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><icon>http://example.com/icon.png</icon></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            icon: Some("http://example.com/icon.png".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_logo() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><logo>http://example.com/logo.png</logo></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            logo: Some("http://example.com/logo.png".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_rights() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><rights>All rights reversed.</rights></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            rights: Some("All rights reversed.".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_subtitle() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><subtitle>An example in the domain of Internet domains</subtitle></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            subtitle: Some("An example in the domain of Internet domains".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_invalid_generator() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><generator/></feed>").unwrap());
        assert_eq!(feed, Err("<generator> is missing required name"));
    }

    #[test]
    fn from_xml_with_generator() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><generator>Atom Feed Generator Deluxe</generator></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            generator: Some(Generator {
                name: "Atom Feed Generator Deluxe".to_string(),
                ..Generator::default()
            }),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_full_generator() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><generator uri='https://atom-feed-generator-deluxe.example/' version='2.0'>Atom Feed Generator Deluxe</generator></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            generator: Some(Generator {
                name: "Atom Feed Generator Deluxe".to_string(),
                uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
                version: Some("2.0".to_string()),
            }),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_invalid_link() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><link/></feed>").unwrap());
        assert_eq!(feed, Err(r#"<link> is missing required "href" attribute"#));
    }

    #[test]
    fn from_xml_with_one_link() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><link href='http://example.com/'/></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><link href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><link href='http://example.com/'/><link href='http://example.net/'/><link href='http://example.org/'/></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><category/></feed>").unwrap());
        assert_eq!(feed, Err(r#"<category> is missing required "term" attribute"#));
    }

    #[test]
    fn from_xml_with_one_category() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><category term='announcements'/></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><category term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><category term='announcements'/><category term='news'/><category term='releases'/></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><author/></feed>").unwrap());
        assert_eq!(feed, Err("<author> is missing required <name> element"));
    }

    #[test]
    fn from_xml_with_one_author() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><author><name>John Doe</name></author></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><author><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></author></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><author><name>John Doe</name></author><author><name>Mary Smith</name></author><author><name>Mustapha Ibrahim</name></author></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><contributor/></feed>").unwrap());
        assert_eq!(feed, Err("<contributor> is missing required <name> element"));
    }

    #[test]
    fn from_xml_with_one_contributor() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><contributor><name>Corey Farwell</name></contributor></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><contributor><name>Corey Farwell</name><uri>https://rwell.org/</uri><email>coreyf@rwell.org</email></contributor></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><contributor><name>Corey Farwell</name></contributor><contributor><name>Duncan</name></contributor><contributor><name>Francis Gagné</name></contributor></feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
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
    fn from_xml_with_invalid_entry() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id><title>Examplar Feed</title><updated>2016-09-18T18:53:16Z</updated><entry><title>First!</title><updated>2016-09-17T19:18:32Z</updated></entry></feed>").unwrap());
        assert_eq!(feed, Err("<entry> is missing required <id> element"));
    }

    #[test]
    fn from_xml_with_one_entry() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'>
            <id>http://example.com/feed.atom</id>
            <title>Examplar Feed</title>
            <updated>2016-09-18T18:53:16Z</updated>
            <entry>
                <id>http://example.com/1</id>
                <title>First!</title>
                <updated>2016-09-17T19:18:32Z</updated>
            </entry>
        </feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            entries: vec![
                Entry {
                    id: "http://example.com/1".to_string(),
                    title: "First!".to_string(),
                    updated: "2016-09-17T19:18:32Z".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_one_full_entry() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'>
            <id>http://example.com/feed.atom</id>
            <title>Examplar Feed</title>
            <updated>2016-09-18T18:53:16Z</updated>
            <entry>
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
                <link href='http://example.com/'/>
                <category term='announcements'/>
                <author>
                    <name>John Doe</name>
                </author>
                <contributor>
                    <name>Corey Farwell</name>
                </contributor>
                <summary>Summary of the first post.</summary>
                <content>Content of the first post.</content>
            </entry>
        </feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            entries: vec![
                Entry {
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
                    summary: Some("Summary of the first post.".to_string()),
                    content: Some(Content::Text("Content of the first post.".to_string())),
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_several_entries() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'>
            <id>http://example.com/feed.atom</id>
            <title>Examplar Feed</title>
            <updated>2016-09-18T18:53:16Z</updated>
            <entry>
                <id>http://example.com/3</id>
                <title>Third!</title>
                <updated>2016-09-18T18:53:16Z</updated>
            </entry>
            <entry>
                <id>http://example.com/2</id>
                <title>Second!</title>
                <updated>2016-09-18T11:22:55Z</updated>
            </entry>
            <entry>
                <id>http://example.com/1</id>
                <title>First!</title>
                <updated>2016-09-17T19:18:32Z</updated>
            </entry>
        </feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Examplar Feed".to_string(),
            updated: "2016-09-18T18:53:16Z".to_string(),
            entries: vec![
                Entry {
                    id: "http://example.com/3".to_string(),
                    title: "Third!".to_string(),
                    updated: "2016-09-18T18:53:16Z".to_string(),
                    ..Default::default()
                },
                Entry {
                    id: "http://example.com/2".to_string(),
                    title: "Second!".to_string(),
                    updated: "2016-09-18T11:22:55Z".to_string(),
                    ..Default::default()
                },
                Entry {
                    id: "http://example.com/1".to_string(),
                    title: "First!".to_string(),
                    updated: "2016-09-17T19:18:32Z".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_everything() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'>
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
            <entry>
                <id>http://example.com/3</id>
                <title>Third!</title>
                <updated>2016-09-18T18:53:16Z</updated>
                <published>2016-09-18T18:53:16Z</published>
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
            </entry>
            <entry>
                <id>http://example.com/2</id>
                <title>Second!</title>
                <updated>2016-09-18T11:22:55Z</updated>
            </entry>
            <entry>
                <id>http://example.com/1</id>
                <title>First!</title>
                <updated>2016-09-17T19:18:32Z</updated>
            </entry>
        </feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Example Domain Updates".to_string(),
            updated: "2016-08-07T14:13:49Z".to_string(),
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
            entries: vec![
                Entry {
                    id: "http://example.com/3".to_string(),
                    title: "Third!".to_string(),
                    updated: "2016-09-18T18:53:16Z".to_string(),
                    published: Some("2016-09-18T18:53:16Z".to_string()),
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
                },
                Entry {
                    id: "http://example.com/2".to_string(),
                    title: "Second!".to_string(),
                    updated: "2016-09-18T11:22:55Z".to_string(),
                    ..Default::default()
                },
                Entry {
                    id: "http://example.com/1".to_string(),
                    title: "First!".to_string(),
                    updated: "2016-09-17T19:18:32Z".to_string(),
                    ..Default::default()
                },
            ],
        }));
    }

    #[test]
    fn from_xml_with_everything_shuffled() {
        let feed = Feed::from_xml(&str::parse("<feed xmlns='http://www.w3.org/2005/Atom'>
            <contributor>
                <name>Corey Farwell</name>
                <uri>https://rwell.org/</uri>
                <email>coreyf@rwell.org</email>
            </contributor>
            <entry>
                <id>http://example.com/3</id>
                <title>Third!</title>
                <updated>2016-09-18T18:53:16Z</updated>
                <published>2016-09-18T18:53:16Z</published>
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
            </entry>
            <link href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/>
            <title>Example Domain Updates</title>
            <generator uri='https://atom-feed-generator-deluxe.example/' version='2.0'>Atom Feed Generator Deluxe</generator>
            <entry>
                <id>http://example.com/2</id>
                <title>Second!</title>
                <updated>2016-09-18T11:22:55Z</updated>
            </entry>
            <subtitle>An example in the domain of Internet domains</subtitle>
            <contributor>
                <name>Duncan</name>
            </contributor>
            <category term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/>
            <entry>
                <id>http://example.com/1</id>
                <title>First!</title>
                <updated>2016-09-17T19:18:32Z</updated>
            </entry>
            <link href='http://example.net/'/>
            <icon>http://example.com/icon.png</icon>
            <author>
                <name>John Doe</name>
                <uri>http://john.doe.example/</uri>
                <email>john@john.doe.example</email>
            </author>
            <author>
                <name>Mary Smith</name>
            </author>
            <link href='http://example.org/'/>
            <category term='news'/>
            <logo>http://example.com/logo.png</logo>
            <contributor>
                <name>Francis Gagné</name>
            </contributor>
            <category term='releases'/>
            <updated>2016-08-07T14:13:49Z</updated>
            <author>
                <name>Mustapha Ibrahim</name>
            </author>
            <id>http://example.com/feed.atom</id>
            <rights>All rights reversed.</rights>
        </feed>").unwrap());
        assert_eq!(feed, Ok(Feed {
            id: "http://example.com/feed.atom".to_string(),
            title: "Example Domain Updates".to_string(),
            updated: "2016-08-07T14:13:49Z".to_string(),
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
            entries: vec![
                Entry {
                    id: "http://example.com/3".to_string(),
                    title: "Third!".to_string(),
                    updated: "2016-09-18T18:53:16Z".to_string(),
                    published: Some("2016-09-18T18:53:16Z".to_string()),
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
                },
                Entry {
                    id: "http://example.com/2".to_string(),
                    title: "Second!".to_string(),
                    updated: "2016-09-18T11:22:55Z".to_string(),
                    ..Default::default()
                },
                Entry {
                    id: "http://example.com/1".to_string(),
                    title: "First!".to_string(),
                    updated: "2016-09-17T19:18:32Z".to_string(),
                    ..Default::default()
                },
            ],
        }));
    }

    #[test]
    fn from_str() {
        let atom_string = include_str!("../test-data/xkcd.xml");
        let feed = Feed::from_str(atom_string).unwrap();
        assert!(feed.to_string().len() > 0);
    }

    #[test]
    fn from_str_no_feeds() {
        let atom_str = "";
        assert!(Feed::from_str(atom_str).is_err());
    }

    #[test]
    fn from_str_one_feed_no_properties() {
        let atom_str = "\
            <feed>\
            </feed>";
        assert!(Feed::from_str(atom_str).is_err());
    }

    #[test]
    fn from_str_one_feed() {
        let atom_str = r#"
            <feed xmlns="http://www.w3.org/2005/Atom">
                <id></id>
                <title>Hello world!</title>
                <updated></updated>
                <description></description>
            </feed>"#;
        println!("{}", atom_str);
        let feed = Feed::from_str(atom_str).unwrap();
        assert_eq!("Hello world!", feed.title);
    }

    // Ensure reader ignores the PI XML node and continues to parse the feed
    #[test]
    fn from_str_with_pinode() {
        let atom_str = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom">
                <id></id>
                <title>Title</title>
                <updated></updated>
                <description></description>
            </feed>"#;
        let feed = Feed::from_str(atom_str).unwrap();
        assert_eq!("Title", feed.title);
    }

    #[test]
    fn to_string() {
        let feed = Feed::default();
        assert_eq!(feed.to_string(), r#"<?xml version="1.0" encoding="utf-8"?><feed xmlns='http://www.w3.org/2005/Atom'><id></id><title></title><updated></updated></feed>"#);
    }
}
