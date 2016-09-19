use std::convert::AsRef;

use xml::Element;

use NS;
use utils::{ElementUtils, ViaXml};


/// [The Atom Syndication Format § The "atom:generator" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.4)
#[derive(Clone, Default)]
pub struct Generator {
    pub name: String,
    pub uri: Option<String>,
    pub version: Option<String>,
}


impl ViaXml for Generator {
    fn to_xml(&self) -> Element {
        let mut link = Element::new("generator".to_string(), Some(NS.to_string()), vec![]);

        link.text(self.name.clone());

        link.attribute_with_optional_text("uri", &self.uri);
        link.attribute_with_optional_text("version", &self.version);

        link
    }

    fn from_xml(elem: Element) -> Result<Self, &'static str> {
        let name = match elem.content_str().as_ref() {
            "" => return Err(r#"<generator> is missing required name"#),
            n => n.to_string(),
        };

        let uri = elem.get_attribute("uri", None).map(String::from);
        let version = elem.get_attribute("version", None).map(String::from);

        Ok(Generator {
            name: name,
            uri: uri,
            version: version,
        })
    }
}
