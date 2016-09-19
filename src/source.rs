use xml::Element;

use {Category, Generator, Link, NS, Person};
use author::Author;
use contributor::Contributor;
use utils::{ElementUtils, Flip, FromXml, ToXml};


/// [The Atom Syndication Format § The "atom:source" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.11)
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Source {
    pub id: Option<String>,
    pub title: Option<String>,
    pub updated: Option<String>,
    pub icon: Option<String>,
    pub logo: Option<String>,
    pub rights: Option<String>,
    pub subtitle: Option<String>,
    pub generator: Option<Generator>,
    pub links: Vec<Link>,
    pub categories: Vec<Category>,
    pub authors: Vec<Person>,
    pub contributors: Vec<Person>,
}


impl ToXml for Source {
    fn to_xml(&self) -> Element {
        let mut elem = Element::new("source".to_string(), Some(NS.to_string()), vec![]);

        elem.tag_with_optional_text("id", &self.id);
        elem.tag_with_optional_text("title", &self.title);
        elem.tag_with_optional_text("updated", &self.updated);
        elem.tag_with_optional_text("icon", &self.icon);
        elem.tag_with_optional_text("logo", &self.logo);
        elem.tag_with_optional_text("rights", &self.rights);
        elem.tag_with_optional_text("subtitle", &self.subtitle);

        if let Some(ref g) = self.generator {
            elem.tag(g.to_xml());
        }

        for link in &self.links {
            elem.tag(link.to_xml());
        }

        for category in &self.categories {
            elem.tag(category.to_xml());
        }

        for person in &self.authors {
            elem.tag(Author(person).to_xml());
        }

        for person in &self.contributors {
            elem.tag(Contributor(person).to_xml());
        }

        elem
    }
}


impl FromXml for Source {
    fn from_xml(elem: &Element) -> Result<Self, &'static str> {
        let id = elem.get_child("id", Some(NS)).map(Element::content_str);
        let title = elem.get_child("title", Some(NS)).map(Element::content_str);
        let updated = elem.get_child("updated", Some(NS)).map(Element::content_str);
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

        Ok(Source {
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
        })
    }
}


#[cfg(test)]
mod tests {
    use std::str;

    use xml::Element;

    use {Category, Generator, Link, NS, Person, Source};
    use utils::{FromXml, ToXml};

    #[test]
    fn to_xml_empty() {
        let source = Source::default();
        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'/>");
    }

    #[test]
    fn to_xml_with_id() {
        let source = Source {
            id: Some("http://example.com/feed.atom".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id></source>");
    }

    #[test]
    fn to_xml_with_title() {
        let source = Source {
            title: Some("Example Domain Updates".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><title>Example Domain Updates</title></source>");
    }

    #[test]
    fn to_xml_with_updated() {
        let source = Source {
            updated: Some("2016-08-07T14:13:49Z".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><updated>2016-08-07T14:13:49Z</updated></source>");
    }

    #[test]
    fn to_xml_with_icon() {
        let source = Source {
            icon: Some("http://example.com/icon.png".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><icon>http://example.com/icon.png</icon></source>");
    }

    #[test]
    fn to_xml_with_logo() {
        let source = Source {
            logo: Some("http://example.com/logo.png".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><logo>http://example.com/logo.png</logo></source>");
    }

    #[test]
    fn to_xml_with_rights() {
        let source = Source {
            rights: Some("All rights reversed.".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><rights>All rights reversed.</rights></source>");
    }

    #[test]
    fn to_xml_with_subtitle() {
        let source = Source {
            subtitle: Some("An example in the domain of Internet domains".to_string()),
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><subtitle>An example in the domain of Internet domains</subtitle></source>");
    }

    #[test]
    fn to_xml_with_generator() {
        let source = Source {
            generator: Some(Generator {
                name: "Atom Feed Generator Deluxe".to_string(),
                ..Generator::default()
            }),
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><generator>Atom Feed Generator Deluxe</generator></source>");
    }

    #[test]
    fn to_xml_with_full_generator() {
        let source = Source {
            generator: Some(Generator {
                name: "Atom Feed Generator Deluxe".to_string(),
                uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
                version: Some("2.0".to_string()),
            }),
            ..Default::default()
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = source.to_xml();
        let mut expected_element = Element::new(
            "source".to_string(),
            Some(NS.to_string()),
            vec![]);
        let mut generator_element = Element::new(
            "generator".to_string(),
            Some(NS.to_string()),
            vec![
                ("uri".to_string(), None, "https://atom-feed-generator-deluxe.example/".to_string()),
                ("version".to_string(), None, "2.0".to_string()),
            ]);
        generator_element.text("Atom Feed Generator Deluxe".to_string());
        expected_element.tag_stay(generator_element);
        assert_eq!(element, expected_element);
    }

    #[test]
    fn to_xml_with_one_link() {
        let source = Source {
            links: vec![
                Link {
                    href: "http://example.com/".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><link href='http://example.com/'/></source>");
    }

    #[test]
    fn to_xml_with_one_full_link() {
        let source = Source {
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
        let element = source.to_xml();
        assert_eq!(&element,
            Element::new(
                "source".to_string(),
                Some(NS.to_string()),
                vec![])
            .tag_stay(Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.com/".to_string()),
                    ("rel".to_string(), None, "alternate".to_string()),
                    ("type".to_string(), None, "text/html".to_string()),
                    ("hreflang".to_string(), None, "en-US".to_string()),
                    ("title".to_string(), None, "Example Domain".to_string()),
                    ("length".to_string(), None, "606".to_string()),
                ])));
    }

    #[test]
    fn to_xml_with_several_links() {
        let source = Source {
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

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><link href='http://example.com/'/><link href='http://example.net/'/><link href='http://example.org/'/></source>");
    }

    #[test]
    fn to_xml_with_one_category() {
        let source = Source {
            categories: vec![
                Category {
                    term: "announcements".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><category term='announcements'/></source>");
    }

    #[test]
    fn to_xml_with_one_full_category() {
        let source = Source {
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
        let element = source.to_xml();
        assert_eq!(&element,
            Element::new(
                "source".to_string(),
                Some(NS.to_string()),
                vec![])
            .tag_stay(Element::new(
                "category".to_string(),
                Some(NS.to_string()),
                vec![
                    ("term".to_string(), None, "announcements".to_string()),
                    ("scheme".to_string(), None, "http://scheme.example/categorization".to_string()),
                    ("label".to_string(), None, "Announcements".to_string()),
                ])));
    }

    #[test]
    fn to_xml_with_several_categories() {
        let source = Source {
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

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><category term='announcements'/><category term='news'/><category term='releases'/></source>");
    }

    #[test]
    fn to_xml_with_one_author() {
        let source = Source {
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><author><name>John Doe</name></author></source>");
    }

    #[test]
    fn to_xml_with_one_full_author() {
        let source = Source {
            authors: vec![
                Person {
                    name: "John Doe".to_string(),
                    uri: Some("http://john.doe.example/".to_string()),
                    email: Some("john@john.doe.example".to_string()),
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><author><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></author></source>");
    }

    #[test]
    fn to_xml_with_several_authors() {
        let source = Source {
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

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><author><name>John Doe</name></author><author><name>Mary Smith</name></author><author><name>Mustapha Ibrahim</name></author></source>");
    }

    #[test]
    fn to_xml_with_one_contributor() {
        let source = Source {
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><contributor><name>Corey Farwell</name></contributor></source>");
    }

    #[test]
    fn to_xml_with_one_full_contributor() {
        let source = Source {
            contributors: vec![
                Person {
                    name: "Corey Farwell".to_string(),
                    uri: Some("https://rwell.org/".to_string()),
                    email: Some("coreyf@rwell.org".to_string()),
                },
            ],
            ..Default::default()
        };

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><contributor><name>Corey Farwell</name><uri>https://rwell.org/</uri><email>coreyf@rwell.org</email></contributor></source>");
    }

    #[test]
    fn to_xml_with_several_contributors() {
        let source = Source {
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

        let xml = format!("{}", source.to_xml());
        assert_eq!(xml, "<source xmlns='http://www.w3.org/2005/Atom'><contributor><name>Corey Farwell</name></contributor><contributor><name>Duncan</name></contributor><contributor><name>Francis Gagné</name></contributor></source>");
    }

    #[test]
    fn to_xml_with_everything() {
        let source = Source {
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
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = source.to_xml();
        let mut expected_element = Element::new(
            "source".to_string(),
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
            .tag_stay(contributor3_element);
        assert_eq!(element, expected_element);
    }

    #[test]
    fn from_xml_empty() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'/>").unwrap());
        assert_eq!(source, Ok(Source::default()));
    }

    #[test]
    fn from_xml_with_id() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><id>http://example.com/feed.atom</id></source>").unwrap());
        assert_eq!(source, Ok(Source {
            id: Some("http://example.com/feed.atom".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_title() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><title>Example Domain Updates</title></source>").unwrap());
        assert_eq!(source, Ok(Source {
            title: Some("Example Domain Updates".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_updated() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><updated>2016-08-07T14:13:49Z</updated></source>").unwrap());
        assert_eq!(source, Ok(Source {
            updated: Some("2016-08-07T14:13:49Z".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_icon() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><icon>http://example.com/icon.png</icon></source>").unwrap());
        assert_eq!(source, Ok(Source {
            icon: Some("http://example.com/icon.png".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_logo() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><logo>http://example.com/logo.png</logo></source>").unwrap());
        assert_eq!(source, Ok(Source {
            logo: Some("http://example.com/logo.png".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_rights() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><rights>All rights reversed.</rights></source>").unwrap());
        assert_eq!(source, Ok(Source {
            rights: Some("All rights reversed.".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_subtitle() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><subtitle>An example in the domain of Internet domains</subtitle></source>").unwrap());
        assert_eq!(source, Ok(Source {
            subtitle: Some("An example in the domain of Internet domains".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_invalid_generator() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><generator/></source>").unwrap());
        assert_eq!(source, Err("<generator> is missing required name"));
    }

    #[test]
    fn from_xml_with_generator() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><generator>Atom Feed Generator Deluxe</generator></source>").unwrap());
        assert_eq!(source, Ok(Source {
            generator: Some(Generator {
                name: "Atom Feed Generator Deluxe".to_string(),
                ..Generator::default()
            }),
            ..Default::default()
        }));
    }

    #[test]
    fn from_xml_with_full_generator() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><generator uri='https://atom-feed-generator-deluxe.example/' version='2.0'>Atom Feed Generator Deluxe</generator></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><link/></source>").unwrap());
        assert_eq!(source, Err(r#"<link> is missing required "href" attribute"#));
    }

    #[test]
    fn from_xml_with_one_link() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><link href='http://example.com/'/></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><link href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><link href='http://example.com/'/><link href='http://example.net/'/><link href='http://example.org/'/></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><category/></source>").unwrap());
        assert_eq!(source, Err(r#"<category> is missing required "term" attribute"#));
    }

    #[test]
    fn from_xml_with_one_category() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><category term='announcements'/></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><category term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><category term='announcements'/><category term='news'/><category term='releases'/></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><author/></source>").unwrap());
        assert_eq!(source, Err("<author> is missing required <name> element"));
    }

    #[test]
    fn from_xml_with_one_author() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><author><name>John Doe</name></author></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><author><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></author></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><author><name>John Doe</name></author><author><name>Mary Smith</name></author><author><name>Mustapha Ibrahim</name></author></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><contributor/></source>").unwrap());
        assert_eq!(source, Err("<contributor> is missing required <name> element"));
    }

    #[test]
    fn from_xml_with_one_contributor() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><contributor><name>Corey Farwell</name></contributor></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><contributor><name>Corey Farwell</name><uri>https://rwell.org/</uri><email>coreyf@rwell.org</email></contributor></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'><contributor><name>Corey Farwell</name></contributor><contributor><name>Duncan</name></contributor><contributor><name>Francis Gagné</name></contributor></source>").unwrap());
        assert_eq!(source, Ok(Source {
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
    fn from_xml_with_everything() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'>
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
        </source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        }));
    }

    #[test]
    fn from_xml_with_everything_shuffled() {
        let source = Source::from_xml(&str::parse("<source xmlns='http://www.w3.org/2005/Atom'>
            <author><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></author>
            <category term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/>
            <contributor><name>Corey Farwell</name><uri>https://rwell.org/</uri><email>coreyf@rwell.org</email></contributor>
            <icon>http://example.com/icon.png</icon>
            <generator uri='https://atom-feed-generator-deluxe.example/' version='2.0'>Atom Feed Generator Deluxe</generator>
            <updated>2016-08-07T14:13:49Z</updated>
            <contributor><name>Duncan</name></contributor>
            <logo>http://example.com/logo.png</logo>
            <category term='news'/>
            <link href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/>
            <title>Example Domain Updates</title>
            <id>http://example.com/feed.atom</id>
            <contributor><name>Francis Gagné</name></contributor>
            <link href='http://example.net/'/>
            <subtitle>An example in the domain of Internet domains</subtitle>
            <rights>All rights reversed.</rights>
            <author><name>Mary Smith</name></author>
            <link href='http://example.org/'/>
            <category term='releases'/>
            <author><name>Mustapha Ibrahim</name></author>
        </source>").unwrap());
        assert_eq!(source, Ok(Source {
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
        }));
    }
}
