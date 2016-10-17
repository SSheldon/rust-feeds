use xml::{Xml, Element};

use {NS, XHTML_NS};
use utils::{FromXml, ToXml};

/// [The Atom Syndication Format § The "atom:content" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.1.3)
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Text(String),
    Html(String),
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

