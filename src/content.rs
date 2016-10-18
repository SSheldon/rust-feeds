use xml::{Xml, Element};

use {NS, XHTML_NS};
use utils::{FromXml, ToXml};

/// [The Atom Syndication Format § The "atom:content" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.1.3)
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    /// Plain text only, no markup
    Text(String),
    /// String containing escaped HTML markup
    Html(String),
    /// XHTML div element embedded in the feed
    Xhtml(Element),
}

impl Content {
    fn type_string(&self) -> String {
        match *self {
            Content::Text(_) => "text".to_string(),
            Content::Html(_) => "html".to_string(),
            Content::Xhtml(_) => "xhtml".to_string(),
        }
    }
}

impl ToXml for Content {
    fn to_xml(&self) -> Element {
        let type_attr = ("type".to_string(), None, self.type_string());
        let mut content = Element::new("content".to_string(), Some(NS.to_string()), vec![type_attr]);
        match *self {
            Content::Text(ref text) | Content::Html(ref text) => { content.text(text.clone()); },
            Content::Xhtml(ref element) => { content.children.push(Xml::ElementNode(element.clone())); },
        };

        content
    }
}

impl FromXml for Content {
    fn from_xml(elem: &Element) -> Result<Self, &'static str> {
        let text = elem.content_str();
        match elem.get_attribute("type", None) {
            Some("text") | None => Ok(Content::Text(text)),
            Some("html") => Ok(Content::Html(text)),
            Some("xhtml") => {
                // https://tools.ietf.org/html/rfc4287#section-4.1.3.3
                // 4.1.3.3.3 If the value of "type" is "xhtml", the content of atom:content MUST be
                // a single XHTML div element
                if let Some(div) = elem.get_child("div", Some(XHTML_NS)) {
                    return Ok(Content::Xhtml(div.clone()));
                }

                Err("expected to find child element <div> of <content> but found none")
            },
            Some(_) => Err("<content> has unknown type")
        }
    }
}

#[cfg(test)]
mod tests {
    use xml::{Xml, Element};

    use {Content, XHTML_NS};
    use utils::{FromXml, ToXml};

    #[test]
    fn to_xml_with_text() {
        let content = Content::Text("Content of the first post.".to_string());
        let xml = format!("{}", content.to_xml());
        assert_eq!(xml, "<content xmlns='http://www.w3.org/2005/Atom' type='text'>Content of the first post.</content>");
    }

    #[test]
    fn to_xml_with_html() {
        let content = Content::Html("<p>Content of the first post.</p>".to_string());
        let xml = format!("{}", content.to_xml());
        assert_eq!(xml, "<content xmlns='http://www.w3.org/2005/Atom' type='html'>&lt;p&gt;Content of the first post.&lt;/p&gt;</content>");
    }

    #[test]
    fn to_xml_with_xhtml() {
        let mut div = Element::new("div".to_string(), Some(XHTML_NS.to_string()), vec![]);
        let mut p = Element::new("p".to_string(), Some(XHTML_NS.to_string()), vec![]);
        p.text("Content of the first post.".to_string());
        div.children.push(Xml::ElementNode(p));

        let content = Content::Xhtml(div);
        let xml = format!("{}", content.to_xml());
        assert_eq!(xml, "<content xmlns='http://www.w3.org/2005/Atom' type='xhtml'><div xmlns='http://www.w3.org/1999/xhtml'><p>Content of the first post.</p></div></content>");
    }

    #[test]
    fn from_xml_with_text() {
        let content = Content::from_xml(&str::parse("<content xmlns='http://www.w3.org/2005/Atom'>Content of the first post.</content>").unwrap());
        assert_eq!(content, Ok(Content::Text("Content of the first post.".to_string())));
    }

    #[test]
    fn from_xml_with_html() {
        let content = Content::from_xml(&str::parse("<content xmlns='http://www.w3.org/2005/Atom' type='html'>&lt;p&gt;Content of the first post.&lt;/p&gt;</content>").unwrap());
        assert_eq!(content, Ok(Content::Html("<p>Content of the first post.</p>".to_string())));
    }

    #[test]
    fn from_xml_with_xhtml() {
        let content = Content::from_xml(&str::parse("<content xmlns='http://www.w3.org/2005/Atom' type='xhtml'><div xmlns='http://www.w3.org/1999/xhtml'><p>Content of the first post.</p></div></content>").unwrap());

        let namespace_attr = ("xmlns".to_string(), None, "http://www.w3.org/1999/xhtml".to_string());
        let mut div = Element::new("div".to_string(), Some(XHTML_NS.to_string()), vec![namespace_attr]);
        let mut p = Element::new("p".to_string(), Some(XHTML_NS.to_string()), vec![]);
        p.text("Content of the first post.".to_string());
        div.children.push(Xml::ElementNode(p));

        assert_eq!(content, Ok(Content::Xhtml(div)));
    }

    #[test]
    fn from_xml_with_xhtml_content_no_div() {
        let content = Content::from_xml(&str::parse("<content xmlns='http://www.w3.org/2005/Atom' type='xhtml'><p>Content of the first post.</p></content>").unwrap());
        assert_eq!(content, Err("expected to find child element <div> of <content> but found none"));
    }

    #[test]
    fn from_xml_with_invalid_content_type() {
        let content = Content::from_xml(&str::parse("<content xmlns='http://www.w3.org/2005/Atom' type='invalid'>Content of the first post.</content>").unwrap());
        assert_eq!(content, Err("<content> has unknown type"));
    }
}
