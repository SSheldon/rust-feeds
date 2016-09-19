use xml::Element;

use {NS, Person};
use utils::{ElementUtils, ViaXml};


/// [The Atom Syndication Format § The "atom:author" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.1)
#[derive(Clone, Default)]
pub struct Author(pub Person);


impl ViaXml for Author {
    fn to_xml(&self) -> Element {
        let mut elem = Element::new("author".to_string(), Some(NS.to_string()), vec![]);

        let &Author(ref person) = self;

        elem.tag_with_text("name", &person.name);
        elem.tag_with_optional_text("uri", &person.uri);
        elem.tag_with_optional_text("email", &person.email);

        elem
    }

    fn from_xml(elem: Element) -> Result<Self, &'static str> {
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
