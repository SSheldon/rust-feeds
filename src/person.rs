/// [The Atom Syndication Format § Person Constructs]
/// (https://tools.ietf.org/html/rfc4287#section-3.2)
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Person {
    pub name: String,
    pub uri: Option<String>,
    pub email: Option<String>,
}
