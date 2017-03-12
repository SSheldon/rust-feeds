use xml::Element;

use NS;

pub trait ElementUtils {
    fn tag_with_text(&mut self, child_name: &'static str, child_body: &str);
    fn tag_with_optional_text(&mut self, child_name: &'static str, child_body: &Option<String>);
    fn attribute_with_text(&mut self, attribute_name: &'static str, attribute_value: &str);
    fn attribute_with_optional_text(&mut self, attribute_name: &'static str, attribute_value: &Option<String>);
}


impl ElementUtils for Element {
    fn tag_with_text(&mut self, child_name: &'static str, child_body: &str) {
        self.tag(elem_with_text(child_name, child_body));
    }

    fn tag_with_optional_text(&mut self, child_name: &'static str, child_body: &Option<String>) {
        if let Some(ref c) = *child_body {
            self.tag_with_text(child_name, c);
        }
    }

    fn attribute_with_text(&mut self, attribute_name: &'static str, attribute_value: &str) {
        self.set_attribute(attribute_name.to_string(), None, attribute_value.to_string());
    }

    fn attribute_with_optional_text(&mut self, attribute_name: &'static str, attribute_value: &Option<String>) {
        if let Some(ref v) = *attribute_value {
            self.attribute_with_text(attribute_name, v);
        }
    }
}


fn elem_with_text(tag_name: &'static str, chars: &str) -> Element {
    let mut elem = Element::new(tag_name.to_string(), Some(NS.to_string()), vec![]);
    elem.text(chars.to_string());
    elem
}


pub trait ToXml {
    fn to_xml(&self) -> Element;
}


pub trait FromXml: Sized {
    fn from_xml(elem: &Element) -> Result<Self, &'static str>;
}


pub trait Flip {
    type Output;

    fn flip(self) -> Self::Output;
}


impl<T, E> Flip for Option<Result<T, E>> {
    type Output = Result<Option<T>, E>;

    fn flip(self) -> Self::Output {
        match self {
            Some(Ok(x)) => Ok(Some(x)),
            Some(Err(x)) => Err(x),
            None => Ok(None),
        }
    }
}
