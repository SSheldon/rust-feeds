use std::borrow::Borrow;
use xml::Element;

use {NS, Person};
use utils::{ElementUtils, FromXml, ToXml};


/// [The Atom Syndication Format § The "atom:contributor" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.3)
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Contributor<P: Borrow<Person>>(pub P);


impl<P: Borrow<Person>> ToXml for Contributor<P> {
    fn to_xml(&self) -> Element {
        let mut elem = Element::new("contributor".to_string(), Some(NS.to_string()), vec![]);

        let person = self.0.borrow();

        elem.tag_with_text("name", &person.name);
        elem.tag_with_optional_text("uri", &person.uri);
        elem.tag_with_optional_text("email", &person.email);

        elem
    }
}


impl FromXml for Contributor<Person> {
    fn from_xml(elem: &Element) -> Result<Self, &'static str> {
        let name = match elem.get_child("name", Some(NS)) {
            Some(elem) => elem.content_str(),
            None => return Err("<contributor> is missing required <name> element"),
        };

        let uri = elem.get_child("uri", Some(NS)).map(Element::content_str);
        let email = elem.get_child("email", Some(NS)).map(Element::content_str);

        Ok(Contributor(Person {
            name: name,
            uri: uri,
            email: email,
        }))
    }
}


#[cfg(test)]
mod tests {
    use std::str;

    use Person;
    use contributor::Contributor;
    use utils::{FromXml, ToXml};

    #[test]
    fn to_xml_name_only() {
        let contributor = Contributor(Person {
            name: "John Doe".to_string(),
            uri: None,
            email: None,
        });

        let xml = format!("{}", contributor.to_xml());
        assert_eq!(xml, "<contributor xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name></contributor>");
    }

    #[test]
    fn to_xml_with_uri() {
        let contributor = Contributor(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: None,
        });

        let xml = format!("{}", contributor.to_xml());
        assert_eq!(xml, "<contributor xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><uri>http://john.doe.example/</uri></contributor>");
    }

    #[test]
    fn to_xml_with_email() {
        let contributor = Contributor(Person {
            name: "John Doe".to_string(),
            uri: None,
            email: Some("john@john.doe.example".to_string()),
        });

        let xml = format!("{}", contributor.to_xml());
        assert_eq!(xml, "<contributor xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><email>john@john.doe.example</email></contributor>");
    }

    #[test]
    fn to_xml_with_uri_and_email() {
        let contributor = Contributor(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: Some("john@john.doe.example".to_string()),
        });

        let xml = format!("{}", contributor.to_xml());
        assert_eq!(xml, "<contributor xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></contributor>");
    }

    #[test]
    fn from_xml_missing_name() {
        let contributor = Contributor::from_xml(&str::parse("<contributor xmlns='http://www.w3.org/2005/Atom'></contributor>").unwrap());
        assert_eq!(contributor, Err("<contributor> is missing required <name> element"));
    }

    #[test]
    fn from_xml_name_only() {
        let contributor = Contributor::from_xml(&str::parse("<contributor xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name></contributor>").unwrap());
        assert_eq!(contributor, Ok(Contributor(Person {
            name: "John Doe".to_string(),
            uri: None,
            email: None,
        })));
    }

    #[test]
    fn from_xml_with_uri() {
        let contributor = Contributor::from_xml(&str::parse("<contributor xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><uri>http://john.doe.example/</uri></contributor>").unwrap());
        assert_eq!(contributor, Ok(Contributor(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: None,
        })));
    }

    #[test]
    fn from_xml_with_email() {
        let contributor = Contributor::from_xml(&str::parse("<contributor xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><email>john@john.doe.example</email></contributor>").unwrap());
        assert_eq!(contributor, Ok(Contributor(Person {
            name: "John Doe".to_string(),
            uri: None,
            email: Some("john@john.doe.example".to_string()),
        })));
    }

    #[test]
    fn from_xml_with_uri_and_email() {
        let contributor = Contributor::from_xml(&str::parse("<contributor xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></contributor>").unwrap());
        assert_eq!(contributor, Ok(Contributor(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: Some("john@john.doe.example".to_string()),
        })));
    }

    #[test]
    fn from_xml_with_uri_and_email_shuffled() {
        // § Person Constructs: "This specification assigns no significance to the order of appearance of the child elements in a Person construct."
        let contributor = Contributor::from_xml(&str::parse("<contributor xmlns='http://www.w3.org/2005/Atom'><email>john@john.doe.example</email><uri>http://john.doe.example/</uri><name>John Doe</name></contributor>").unwrap());
        assert_eq!(contributor, Ok(Contributor(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: Some("john@john.doe.example".to_string()),
        })));
    }
}
