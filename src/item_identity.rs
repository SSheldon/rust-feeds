use std::borrow::Cow;

fn eq_ignoring_scheme(a: &str, b: &str) -> bool {
    a == b
        || a.strip_prefix("https://").map_or(false, |a| Some(a) == b.strip_prefix("http://"))
        || a.strip_prefix("http://").map_or(false, |a| Some(a) == b.strip_prefix("https://"))
}

#[derive(Clone, Debug)]
pub struct ItemIdentifier<'a> {
    link: Cow<'a, str>,
}

impl<'a> ItemIdentifier<'a> {
    pub fn new(link: Option<&'a str>) -> Option<Self> {
        link.map(|link| Self {
            link: Cow::Borrowed(link),
        })
    }

    pub fn new_owned(link: String) -> Self {
        Self {
            link: Cow::Owned(link),
        }
    }

    pub fn link(&self) -> &str {
        &self.link
    }
}

impl<'a> PartialEq for ItemIdentifier<'a> {
    fn eq(&self, other: &Self) -> bool {
        eq_ignoring_scheme(self.link(), other.link())
    }
}
