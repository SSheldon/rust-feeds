use std::borrow::Cow;

fn eq_ignoring_scheme(a: &str, b: &str) -> bool {
    a == b
        || a.strip_prefix("https://").map_or(false, |a| Some(a) == b.strip_prefix("http://"))
        || a.strip_prefix("http://").map_or(false, |a| Some(a) == b.strip_prefix("https://"))
}

#[derive(Clone, Debug)]
pub struct ItemIdentifier<'a> {
    link: Cow<'a, str>,
    guid: Option<Cow<'a, str>>,
}

impl<'a> ItemIdentifier<'a> {
    pub fn new(link: Option<&'a str>, guid: Option<&'a str>) -> Option<Self> {
        link.map(|link| Self {
            link: Cow::Borrowed(link),
            guid: guid.map(Cow::Borrowed),
        })
    }

    pub fn new_owned(link: String, guid: Option<String>) -> Self {
        Self {
            link: Cow::Owned(link),
            guid: guid.map(Cow::Owned),
        }
    }

    pub fn link(&self) -> &str {
        &self.link
    }

    pub fn guid(&self) -> Option<&str> {
        self.guid.as_deref()
    }
}

impl<'a> PartialEq for ItemIdentifier<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.guid().zip(other.guid()).map_or(false, |(i1, i2)| i1 == i2)
            || eq_ignoring_scheme(self.link(), other.link())
    }
}
