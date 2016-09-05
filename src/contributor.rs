use std::borrow::Borrow;
use xml::Element;

use {NS, Person};
use utils::{ElementUtils, FromXml, ToXml};


/// [The Atom Syndication Format § The "atom:contributor" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.3)
#[derive(Clone, Default)]
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
