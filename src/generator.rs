use std::convert::AsRef;

use xml::Element;

use NS;
use utils::{ElementUtils, FromXml, ToXml};


/// [The Atom Syndication Format § The "atom:generator" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.4)
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Generator {
    pub name: String,
    pub uri: Option<String>,
    pub version: Option<String>,
}


impl ToXml for Generator {
    fn to_xml(&self) -> Element {
        let mut generator = Element::new("generator".to_string(), Some(NS.to_string()), vec![]);

        generator.text(self.name.clone());

        generator.attribute_with_optional_text("uri", &self.uri);
        generator.attribute_with_optional_text("version", &self.version);

        generator
    }
}


impl FromXml for Generator {
    fn from_xml(elem: &Element) -> Result<Self, &'static str> {
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


#[cfg(test)]
mod tests {
    use std::str;

    use xml::Element;

    use {Generator, NS};
    use utils::{FromXml, ToXml};

    #[test]
    fn to_xml_name_only() {
        let generator = Generator {
            name: "Atom Feed Generator Deluxe".to_string(),
            uri: None,
            version: None,
        };

        let xml = format!("{}", generator.to_xml());
        assert_eq!(xml, "<generator xmlns='http://www.w3.org/2005/Atom'>Atom Feed Generator Deluxe</generator>");
    }

    #[test]
    fn to_xml_with_uri() {
        let generator = Generator {
            name: "Atom Feed Generator Deluxe".to_string(),
            uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
            version: None,
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = generator.to_xml();
        let mut expected_element = Element::new(
            "generator".to_string(),
            Some(NS.to_string()),
            vec![
                ("uri".to_string(), None, "https://atom-feed-generator-deluxe.example/".to_string()),
            ]);
        expected_element.text("Atom Feed Generator Deluxe".to_string());
        assert_eq!(element, expected_element);
    }

    #[test]
    fn to_xml_with_uri_and_version() {
        let generator = Generator {
            name: "Atom Feed Generator Deluxe".to_string(),
            uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
            version: Some("2.0".to_string()),
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = generator.to_xml();
        let mut expected_element = Element::new(
            "generator".to_string(),
            Some(NS.to_string()),
            vec![
                ("uri".to_string(), None, "https://atom-feed-generator-deluxe.example/".to_string()),
                ("version".to_string(), None, "2.0".to_string()),
            ]);
        expected_element.text("Atom Feed Generator Deluxe".to_string());
        assert_eq!(element, expected_element);
    }

    #[test]
    fn from_xml_missing_name() {
        let generator = Generator::from_xml(&str::parse("<generator xmlns='http://www.w3.org/2005/Atom'></generator>").unwrap());
        assert_eq!(generator, Err(r#"<generator> is missing required name"#));
    }

    #[test]
    fn from_xml_name_only() {
        let generator = Generator::from_xml(&str::parse("<generator xmlns='http://www.w3.org/2005/Atom'>Atom Feed Generator Deluxe</generator>").unwrap());
        assert_eq!(generator, Ok(Generator {
            name: "Atom Feed Generator Deluxe".to_string(),
            uri: None,
            version: None,
        }));
    }

    #[test]
    fn from_xml_with_uri() {
        let generator = Generator::from_xml(&str::parse("<generator xmlns='http://www.w3.org/2005/Atom' uri='https://atom-feed-generator-deluxe.example/'>Atom Feed Generator Deluxe</generator>").unwrap());
        assert_eq!(generator, Ok(Generator {
            name: "Atom Feed Generator Deluxe".to_string(),
            uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
            version: None,
        }));
    }

    #[test]
    fn from_xml_with_uri_and_version() {
        let generator = Generator::from_xml(&str::parse("<generator xmlns='http://www.w3.org/2005/Atom' uri='https://atom-feed-generator-deluxe.example/' version='2.0'>Atom Feed Generator Deluxe</generator>").unwrap());
        assert_eq!(generator, Ok(Generator {
            name: "Atom Feed Generator Deluxe".to_string(),
            uri: Some("https://atom-feed-generator-deluxe.example/".to_string()),
            version: Some("2.0".to_string()),
        }));
    }
}
