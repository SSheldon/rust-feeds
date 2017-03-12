use xml::Element;

use NS;
use utils::{ElementUtils, FromXml, ToXml};


/// [The Atom Syndication Format § The "atom:category" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.2)
#[derive(Clone, Default, Debug, PartialEq, Eq)]
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
    fn from_xml(elem: &Element) -> Result<Self, &'static str> {
        let term = match elem.get_attribute("term", None) {
            Some(attr) => attr.to_string(),
            None => return Err(r#"<category> is missing required "term" attribute"#),
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


#[cfg(test)]
mod tests {
    use std::str;

    use xml::Element;

    use {Category, NS};
    use utils::{FromXml, ToXml};

    #[test]
    fn to_xml_term_only() {
        let category = Category {
            term: "announcements".to_string(),
            scheme: None,
            label: None,
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = category.to_xml();
        assert_eq!(element,
            Element::new(
                "category".to_string(),
                Some(NS.to_string()),
                vec![
                    ("term".to_string(), None, "announcements".to_string()),
                ]));
    }

    #[test]
    fn to_xml_with_scheme() {
        let category = Category {
            term: "announcements".to_string(),
            scheme: Some("http://scheme.example/categorization".to_string()),
            label: None,
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = category.to_xml();
        assert_eq!(element,
            Element::new(
                "category".to_string(),
                Some(NS.to_string()),
                vec![
                    ("term".to_string(), None, "announcements".to_string()),
                    ("scheme".to_string(), None, "http://scheme.example/categorization".to_string()),
                ]));
    }

    #[test]
    fn to_xml_with_scheme_and_label() {
        let category = Category {
            term: "announcements".to_string(),
            scheme: Some("http://scheme.example/categorization".to_string()),
            label: Some("Announcements".to_string()),
        };

        // RustyXML renders attributes in a random order, so we can't compare the rendered XML.
        let element = category.to_xml();
        assert_eq!(element,
            Element::new(
                "category".to_string(),
                Some(NS.to_string()),
                vec![
                    ("term".to_string(), None, "announcements".to_string()),
                    ("scheme".to_string(), None, "http://scheme.example/categorization".to_string()),
                    ("label".to_string(), None, "Announcements".to_string()),
                ]));
    }

    #[test]
    fn from_xml_missing_term() {
        let category = Category::from_xml(&str::parse("<category xmlns='http://www.w3.org/2005/Atom'/>").unwrap());
        assert_eq!(category, Err(r#"<category> is missing required "term" attribute"#));
    }

    #[test]
    fn from_xml_term_only() {
        let category = Category::from_xml(&str::parse("<category xmlns='http://www.w3.org/2005/Atom' term='announcements'/>").unwrap());
        assert_eq!(category, Ok(Category {
            term: "announcements".to_string(),
            scheme: None,
            label: None,
        }));
    }

    #[test]
    fn from_xml_with_scheme() {
        let category = Category::from_xml(&str::parse("<category xmlns='http://www.w3.org/2005/Atom' term='announcements' scheme='http://scheme.example/categorization'/>").unwrap());
        assert_eq!(category, Ok(Category {
            term: "announcements".to_string(),
            scheme: Some("http://scheme.example/categorization".to_string()),
            label: None,
        }));
    }

    #[test]
    fn from_xml_with_scheme_and_label() {
        let category = Category::from_xml(&str::parse("<category xmlns='http://www.w3.org/2005/Atom' term='announcements' scheme='http://scheme.example/categorization' label='Announcements'/>").unwrap());
        assert_eq!(category, Ok(Category {
            term: "announcements".to_string(),
            scheme: Some("http://scheme.example/categorization".to_string()),
            label: Some("Announcements".to_string()),
        }));
    }
}
