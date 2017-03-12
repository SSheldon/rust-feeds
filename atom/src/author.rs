use std::borrow::Borrow;
use xml::Element;

use {NS, Person};
use utils::{ElementUtils, FromXml, ToXml};


/// [The Atom Syndication Format § The "atom:author" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.1)
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Author<P: Borrow<Person>>(pub P);


impl<P: Borrow<Person>> ToXml for Author<P> {
    fn to_xml(&self) -> Element {
        let mut elem = Element::new("author".to_string(), Some(NS.to_string()), vec![]);

        let person = self.0.borrow();

        elem.tag_with_text("name", &person.name);
        elem.tag_with_optional_text("uri", &person.uri);
        elem.tag_with_optional_text("email", &person.email);

        elem
    }
}


impl FromXml for Author<Person> {
    fn from_xml(elem: &Element) -> Result<Self, &'static str> {
        let name = match elem.get_child("name", Some(NS)) {
            Some(elem) => elem.content_str(),
            None => return Err("<author> is missing required <name> element"),
        };

        let uri = elem.get_child("uri", Some(NS)).map(Element::content_str);
        let email = elem.get_child("email", Some(NS)).map(Element::content_str);

        Ok(Author(Person {
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
    use author::Author;
    use utils::{FromXml, ToXml};

    #[test]
    fn to_xml_name_only() {
        let author = Author(Person {
            name: "John Doe".to_string(),
            uri: None,
            email: None,
        });

        let xml = format!("{}", author.to_xml());
        assert_eq!(xml, "<author xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name></author>");
    }

    #[test]
    fn to_xml_with_uri() {
        let author = Author(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: None,
        });

        let xml = format!("{}", author.to_xml());
        assert_eq!(xml, "<author xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><uri>http://john.doe.example/</uri></author>");
    }

    #[test]
    fn to_xml_with_email() {
        let author = Author(Person {
            name: "John Doe".to_string(),
            uri: None,
            email: Some("john@john.doe.example".to_string()),
        });

        let xml = format!("{}", author.to_xml());
        assert_eq!(xml, "<author xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><email>john@john.doe.example</email></author>");
    }

    #[test]
    fn to_xml_with_uri_and_email() {
        let author = Author(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: Some("john@john.doe.example".to_string()),
        });

        let xml = format!("{}", author.to_xml());
        assert_eq!(xml, "<author xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></author>");
    }

    #[test]
    fn from_xml_missing_name() {
        let author = Author::from_xml(&str::parse("<author xmlns='http://www.w3.org/2005/Atom'></author>").unwrap());
        assert_eq!(author, Err("<author> is missing required <name> element"));
    }

    #[test]
    fn from_xml_name_only() {
        let author = Author::from_xml(&str::parse("<author xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name></author>").unwrap());
        assert_eq!(author, Ok(Author(Person {
            name: "John Doe".to_string(),
            uri: None,
            email: None,
        })));
    }

    #[test]
    fn from_xml_with_uri() {
        let author = Author::from_xml(&str::parse("<author xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><uri>http://john.doe.example/</uri></author>").unwrap());
        assert_eq!(author, Ok(Author(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: None,
        })));
    }

    #[test]
    fn from_xml_with_email() {
        let author = Author::from_xml(&str::parse("<author xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><email>john@john.doe.example</email></author>").unwrap());
        assert_eq!(author, Ok(Author(Person {
            name: "John Doe".to_string(),
            uri: None,
            email: Some("john@john.doe.example".to_string()),
        })));
    }

    #[test]
    fn from_xml_with_uri_and_email() {
        let author = Author::from_xml(&str::parse("<author xmlns='http://www.w3.org/2005/Atom'><name>John Doe</name><uri>http://john.doe.example/</uri><email>john@john.doe.example</email></author>").unwrap());
        assert_eq!(author, Ok(Author(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: Some("john@john.doe.example".to_string()),
        })));
    }

    #[test]
    fn from_xml_with_uri_and_email_shuffled() {
        // § Person Constructs: "This specification assigns no significance to the order of appearance of the child elements in a Person construct."
        let author = Author::from_xml(&str::parse("<author xmlns='http://www.w3.org/2005/Atom'><email>john@john.doe.example</email><uri>http://john.doe.example/</uri><name>John Doe</name></author>").unwrap());
        assert_eq!(author, Ok(Author(Person {
            name: "John Doe".to_string(),
            uri: Some("http://john.doe.example/".to_string()),
            email: Some("john@john.doe.example".to_string()),
        })));
    }
}
