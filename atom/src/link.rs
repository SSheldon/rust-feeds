use xml::Element;

use NS;
use utils::{ElementUtils, FromXml, ToXml};


/// [The Atom Syndication Format § The "atom:link" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.7)
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Link {
    pub href: String,
    pub rel: Option<String>,
    pub mediatype: Option<String>,
    pub hreflang: Option<String>,
    pub title: Option<String>,
    pub length: Option<String>,
}


impl ToXml for Link {
    fn to_xml(&self) -> Element {
        let mut link = Element::new("link".to_string(), Some(NS.to_string()), vec![]);

        link.attribute_with_text("href", &self.href);

        link.attribute_with_optional_text("rel", &self.rel);
        link.attribute_with_optional_text("type", &self.mediatype);
        link.attribute_with_optional_text("hreflang", &self.hreflang);
        link.attribute_with_optional_text("title", &self.title);
        link.attribute_with_optional_text("length", &self.length);

        link
    }
}


impl FromXml for Link {
    fn from_xml(elem: &Element) -> Result<Self, &'static str> {
        let href = match elem.get_attribute("href", None) {
            Some(attr) => attr.to_string(),
            None => return Err(r#"<link> is missing required "href" attribute"#),
        };

        let rel = elem.get_attribute("rel", None).map(String::from);
        let mediatype = elem.get_attribute("type", None).map(String::from);
        let hreflang = elem.get_attribute("hreflang", None).map(String::from);
        let title = elem.get_attribute("title", None).map(String::from);
        let length = elem.get_attribute("length", None).map(String::from);

        Ok(Link {
            href: href,
            rel: rel,
            mediatype: mediatype,
            hreflang: hreflang,
            title: title,
            length: length,
        })
    }
}


#[cfg(test)]
mod tests {
    use std::str;

    use xml::Element;

    use {Link, NS};
    use utils::{FromXml, ToXml};

    #[test]
    fn to_xml_href_only() {
        let link = Link {
            href: "http://example.com/".to_string(),
            rel: None,
            mediatype: None,
            hreflang: None,
            title: None,
            length: None,
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = link.to_xml();
        assert_eq!(element,
            Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.com/".to_string()),
                ]));
    }

    #[test]
    fn to_xml_with_rel() {
        let link = Link {
            href: "http://example.com/".to_string(),
            rel: Some("related".to_string()),
            mediatype: None,
            hreflang: None,
            title: None,
            length: None,
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = link.to_xml();
        assert_eq!(element,
            Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.com/".to_string()),
                    ("rel".to_string(), None, "related".to_string()),
                ]));
    }

    #[test]
    fn to_xml_with_mediatype() {
        let link = Link {
            href: "http://pictures.example/cat.png".to_string(),
            rel: None,
            mediatype: Some("image/png".to_string()),
            hreflang: None,
            title: None,
            length: None,
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = link.to_xml();
        assert_eq!(element,
            Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://pictures.example/cat.png".to_string()),
                    ("type".to_string(), None, "image/png".to_string()),
                ]));
    }

    #[test]
    fn to_xml_with_hreflang() {
        let link = Link {
            href: "http://example.com/".to_string(),
            rel: None,
            mediatype: None,
            hreflang: Some("en-US".to_string()),
            title: None,
            length: None,
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = link.to_xml();
        assert_eq!(element,
            Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.com/".to_string()),
                    ("hreflang".to_string(), None, "en-US".to_string()),
                ]));
    }

    #[test]
    fn to_xml_with_title() {
        let link = Link {
            href: "http://example.com/".to_string(),
            rel: None,
            mediatype: None,
            hreflang: None,
            title: Some("Example Domain".to_string()),
            length: None,
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = link.to_xml();
        assert_eq!(element,
            Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.com/".to_string()),
                    ("title".to_string(), None, "Example Domain".to_string()),
                ]));
    }

    #[test]
    fn to_xml_with_length() {
        let link = Link {
            href: "http://example.com/".to_string(),
            rel: None,
            mediatype: None,
            hreflang: None,
            title: None,
            length: Some("606".to_string()),
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = link.to_xml();
        assert_eq!(element,
            Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.com/".to_string()),
                    ("length".to_string(), None, "606".to_string()),
                ]));
    }

    #[test]
    fn to_xml_with_all_attributes() {
        let link = Link {
            href: "http://example.com/".to_string(),
            rel: Some("alternate".to_string()),
            mediatype: Some("text/html".to_string()),
            hreflang: Some("en-US".to_string()),
            title: Some("Example Domain".to_string()),
            length: Some("606".to_string()),
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = link.to_xml();
        assert_eq!(element,
            Element::new(
                "link".to_string(),
                Some(NS.to_string()),
                vec![
                    ("href".to_string(), None, "http://example.com/".to_string()),
                    ("rel".to_string(), None, "alternate".to_string()),
                    ("type".to_string(), None, "text/html".to_string()),
                    ("hreflang".to_string(), None, "en-US".to_string()),
                    ("title".to_string(), None, "Example Domain".to_string()),
                    ("length".to_string(), None, "606".to_string()),
                ]));
    }

    #[test]
    fn from_xml_missing_href() {
        let link = Link::from_xml(&str::parse("<link xmlns='http://www.w3.org/2005/Atom'/>").unwrap());
        assert_eq!(link, Err(r#"<link> is missing required "href" attribute"#));
    }

    #[test]
    fn from_xml_href_only() {
        let link = Link::from_xml(&str::parse("<link xmlns='http://www.w3.org/2005/Atom' href='http://example.com/'/>").unwrap());
        assert_eq!(link, Ok(Link {
            href: "http://example.com/".to_string(),
            rel: None,
            mediatype: None,
            hreflang: None,
            title: None,
            length: None,
        }));
    }

    #[test]
    fn from_xml_with_rel() {
        let link = Link::from_xml(&str::parse("<link xmlns='http://www.w3.org/2005/Atom' href='http://example.com/' rel='related'/>").unwrap());
        assert_eq!(link, Ok(Link {
            href: "http://example.com/".to_string(),
            rel: Some("related".to_string()),
            mediatype: None,
            hreflang: None,
            title: None,
            length: None,
        }));
    }

    #[test]
    fn from_xml_with_mediatype() {
        let link = Link::from_xml(&str::parse("<link xmlns='http://www.w3.org/2005/Atom' href='http://pictures.example/cat.png' type='image/png'/>").unwrap());
        assert_eq!(link, Ok(Link {
            href: "http://pictures.example/cat.png".to_string(),
            rel: None,
            mediatype: Some("image/png".to_string()),
            hreflang: None,
            title: None,
            length: None,
        }));
    }

    #[test]
    fn from_xml_with_hreflang() {
        let link = Link::from_xml(&str::parse("<link xmlns='http://www.w3.org/2005/Atom' href='http://example.com/' hreflang='en-US'/>").unwrap());
        assert_eq!(link, Ok(Link {
            href: "http://example.com/".to_string(),
            rel: None,
            mediatype: None,
            hreflang: Some("en-US".to_string()),
            title: None,
            length: None,
        }));
    }

    #[test]
    fn from_xml_with_title() {
        let link = Link::from_xml(&str::parse("<link xmlns='http://www.w3.org/2005/Atom' href='http://example.com/' title='Example Domain'/>").unwrap());
        assert_eq!(link, Ok(Link {
            href: "http://example.com/".to_string(),
            rel: None,
            mediatype: None,
            hreflang: None,
            title: Some("Example Domain".to_string()),
            length: None,
        }));
    }

    #[test]
    fn from_xml_with_length() {
        let link = Link::from_xml(&str::parse("<link xmlns='http://www.w3.org/2005/Atom' href='http://example.com/' length='606'/>").unwrap());
        assert_eq!(link, Ok(Link {
            href: "http://example.com/".to_string(),
            rel: None,
            mediatype: None,
            hreflang: None,
            title: None,
            length: Some("606".to_string()),
        }));
    }

    #[test]
    fn from_xml_with_all_attributes() {
        let link = Link::from_xml(&str::parse("<link xmlns='http://www.w3.org/2005/Atom' href='http://example.com/' rel='alternate' type='text/html' hreflang='en-US' title='Example Domain' length='606'/>").unwrap());
        assert_eq!(link, Ok(Link {
            href: "http://example.com/".to_string(),
            rel: Some("alternate".to_string()),
            mediatype: Some("text/html".to_string()),
            hreflang: Some("en-US".to_string()),
            title: Some("Example Domain".to_string()),
            length: Some("606".to_string()),
        }));
    }
}
