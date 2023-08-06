use std::borrow::Cow;

fn eq_ignoring_scheme(a: &str, b: &str) -> bool {
    a == b
        || a.strip_prefix("https://").map_or(false, |a| Some(a) == b.strip_prefix("http://"))
        || a.strip_prefix("http://").map_or(false, |a| Some(a) == b.strip_prefix("https://"))
}

#[derive(Clone, Debug)]
pub enum ItemIdentifier<'a> {
    Link(Cow<'a, str>),
    Guid(Cow<'a, str>),
    Both(Cow<'a, str>, Cow<'a, str>),
}

impl<'a> ItemIdentifier<'a> {
    pub fn new(link: Option<&'a str>, guid: Option<&'a str>) -> Option<Self> {
        match (link, guid) {
            (Some(link), Some(guid)) => Some(Self::Both(Cow::Borrowed(link), Cow::Borrowed(guid))),
            (Some(link), None) => Some(Self::Link(Cow::Borrowed(link))),
            (None, Some(guid)) => Some(Self::Guid(Cow::Borrowed(guid))),
            (None, None) => None,
        }
    }

    pub fn new_owned(link: Option<String>, guid: Option<String>) -> Option<Self> {
        match (link, guid) {
            (Some(link), Some(guid)) => Some(Self::Both(Cow::Owned(link), Cow::Owned(guid))),
            (Some(link), None) => Some(Self::Link(Cow::Owned(link))),
            (None, Some(guid)) => Some(Self::Guid(Cow::Owned(guid))),
            (None, None) => None,
        }
    }

    pub fn link(&self) -> Option<&str> {
        match self {
            Self::Link(link) => Some(link),
            Self::Guid(_) => None,
            Self::Both(link, _) => Some(link),
        }
    }

    pub fn guid(&self) -> Option<&str> {
        match self {
            Self::Link(_) => None,
            Self::Guid(guid) => Some(guid),
            Self::Both(_, guid) => Some(guid),
        }
    }
}

impl<'a> PartialEq for ItemIdentifier<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.guid().zip(other.guid()).map_or(false, |(i1, i2)| i1 == i2)
            || self.link().zip(other.link()).map_or(false, |(l1, l2)| eq_ignoring_scheme(l1, l2))
    }
}
