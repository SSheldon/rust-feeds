use atom_syndication::Entry as AtomEntry;
use rss::Item as RssItem;

enum EntryKind {
    Rss(RssItem),
    Atom(AtomEntry),
}

pub struct Entry {
    kind: EntryKind,
}
