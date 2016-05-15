use xml::Element;

use ::{ElementUtils, NS, ViaXml};


/// [The Atom Syndication Format § The "atom:link" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.7)
#[derive(Clone, Default)]
pub struct Link {
    pub href: String,
    pub rel: Option<String>,
    pub mediatype: Option<String>,
    pub hreflang: Option<String>,
    pub title: Option<String>,
    pub length: Option<String>,
}


impl ViaXml for Link {
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

    fn from_xml(elem: Element) -> Result<Self, &'static str> {
        let href = match elem.get_attribute("href", None) {
            Some(attr) => attr.to_string(),
            None => return Err(r#"<link> is missing required "href" element"#),
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
