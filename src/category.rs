use xml::Element;

use NS;
use utils::{ElementUtils, FromXml, ToXml};


/// [The Atom Syndication Format § The "atom:category" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.2)
#[derive(Clone, Default)]
pub struct Category {
    pub term: String,
    pub scheme: Option<String>,
    pub label: Option<String>,
}


impl ToXml for Category {
    fn to_xml(&self) -> Element {
        let mut link = Element::new("category".to_string(), Some(NS.to_string()), vec![]);

        link.attribute_with_text("term", &self.term);

        link.attribute_with_optional_text("scheme", &self.scheme);
        link.attribute_with_optional_text("label", &self.label);

        link
    }
}


impl FromXml for Category {
    fn from_xml(elem: Element) -> Result<Self, &'static str> {
        let term = match elem.get_attribute("term", None) {
            Some(attr) => attr.to_string(),
            None => return Err(r#"<link> is missing required "term" element"#),
        };

        let scheme = elem.get_attribute("scheme", None).map(String::from);
        let label = elem.get_attribute("label", None).map(String::from);

        Ok(Category {
            term: term,
            scheme: scheme,
            label: label,
        })
    }
}
