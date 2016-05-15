use xml::Element;

use ::{Author, Category, Contributor, ElementUtils, Generator, Link, NS, Person, ViaXml};


/// [The Atom Syndication Format § The "atom:source" Element]
/// (https://tools.ietf.org/html/rfc4287#section-4.2.11)
#[derive(Clone, Default)]
pub struct Source {
    pub id: Option<String>,
    pub title: Option<String>,
    pub updated: Option<String>,
    pub icon: Option<String>,
    pub logo: Option<String>,
    pub rights: Option<String>,
    pub subtitle: Option<String>,
    pub generator: Option<Generator>,
    pub links: Vec<Link>,
    pub categories: Vec<Category>,
    pub authors: Vec<Person>,
    pub contributors: Vec<Person>,
}

impl ViaXml for Source {
    fn to_xml(&self) -> Element {
        let mut elem = Element::new("source".to_string(), Some(NS.to_string()), vec![]);

        elem.tag_with_optional_text("id", &self.id);
        elem.tag_with_optional_text("title", &self.title);
        elem.tag_with_optional_text("updated", &self.updated);
        elem.tag_with_optional_text("icon", &self.icon);
        elem.tag_with_optional_text("logo", &self.logo);
        elem.tag_with_optional_text("rights", &self.rights);
        elem.tag_with_optional_text("subtitle", &self.subtitle);

        if let Some(ref g) = self.generator {
            elem.tag(g.to_xml());
        }

        for link in &self.links {
            elem.tag(link.to_xml());
        }

        for category in &self.categories {
            elem.tag(category.to_xml());
        }

        for person in &self.authors {
            elem.tag(Author(person.clone()).to_xml());
        }

        for person in &self.contributors {
            elem.tag(Contributor(person.clone()).to_xml());
        }

        elem
    }

    fn from_xml(elem: Element) -> Result<Self, &'static str> {
        let id = elem.get_child("id", Some(NS)).map(Element::content_str);
        let title = elem.get_child("title", Some(NS)).map(Element::content_str);
        let updated = elem.get_child("updated", Some(NS)).map(Element::content_str);
        let icon = elem.get_child("icon", Some(NS)).map(Element::content_str);
        let logo = elem.get_child("logo", Some(NS)).map(Element::content_str);
        let rights = elem.get_child("rights", Some(NS)).map(Element::content_str);
        let subtitle = elem.get_child("subtitle", Some(NS)).map(Element::content_str);
        let generator = elem.get_child("generator", Some(NS)).map(|e| ViaXml::from_xml(e.clone()).unwrap());

        let links = elem.get_children("link", Some(NS))
            .map(|e| ViaXml::from_xml(e.clone()).unwrap())
            .collect();

        let categories = elem.get_children("category", Some(NS))
            .map(|e| ViaXml::from_xml(e.clone()).unwrap())
            .collect();

        let authors = elem.get_children("author", Some(NS))
            .map(|e| {
                let Author(person) = ViaXml::from_xml(e.clone()).unwrap();
                person
            }).collect();

        let contributors = elem.get_children("contributor", Some(NS))
            .map(|e| {
                let Contributor(person) = ViaXml::from_xml(e.clone()).unwrap();
                person
            }).collect();

        Ok(Source {
            id: id,
            title: title,
            updated: updated,
            icon: icon,
            logo: logo,
            rights: rights,
            subtitle: subtitle,
            generator: generator,
            links: links,
            categories: categories,
            authors: authors,
            contributors: contributors,
        })
    }
}
