use std::ascii::AsciiExt;
use std::io::Read;

use rss::ReadError;
use xml::{Element, ElementBuilder, EndTag, Event, Parser, ParserError, StartTag};

use entry::{Entry, self};
use str_buf_reader::StrBufReader;

#[derive(Clone, Copy)]
enum ParserState {
    None,
    InRss,
    InChannel,
    InFeed,
}

pub struct FeedParser<R> {
    reader: StrBufReader<R>,
    parser: Parser,
    builder: ElementBuilder,
    state: ParserState,
}

impl<R: Read> FeedParser<R> {
    pub fn new(source: R) -> FeedParser<R> {
        FeedParser {
            reader: StrBufReader::with_capacity(4096, source),
            parser: Parser::new(),
            builder: ElementBuilder::new(),
            state: ParserState::None,
        }
    }

    fn intercept_event(&mut self, event: &Event) -> bool {
        match (self.state, event) {
            (ParserState::None, &Event::ElementStart(StartTag { ref name, .. }))
                    if name.eq_ignore_ascii_case("rss") => {
                self.state = ParserState::InRss;
                true
            }
            (ParserState::None, &Event::ElementStart(StartTag { ref name, .. }))
                    if name.eq_ignore_ascii_case("feed") => {
                self.state = ParserState::InFeed;
                true
            }
            (ParserState::InRss, &Event::ElementStart(StartTag { ref name, .. }))
                    if name.eq_ignore_ascii_case("channel") => {
                self.state = ParserState::InChannel;
                true
            }
            (ParserState::InRss, &Event::ElementEnd(EndTag { ref name, .. }))
                    if name.eq_ignore_ascii_case("rss") => {
                self.state = ParserState::None;
                true
            }
            (ParserState::InChannel, &Event::ElementEnd(EndTag { ref name, .. }))
                    if name.eq_ignore_ascii_case("channel") => {
                self.state = ParserState::InRss;
                true
            }
            (ParserState::InFeed, &Event::ElementEnd(EndTag { ref name, .. }))
                    if name.eq_ignore_ascii_case("feed") => {
                self.state = ParserState::None;
                true
            }
            // Swallow all events until we're in a channel/feed
            (ParserState::None, _) | (ParserState::InRss, _) => true,
            _ => false
        }
    }

    fn next_event(&mut self) -> Option<Result<Event, ParserError>> {
        loop {
            if let Some(event) = self.parser.next() {
                return Some(event);
            }
            match self.reader.next_str() {
                Some(Ok(s)) => {
                    self.parser.feed_str(s);
                }
                _ => return None,
            }
        }
    }

    fn entry_from_element(&self, elem: Element) -> Option<Entry> {
        match self.state {
            ParserState::InChannel if elem.name.eq_ignore_ascii_case("item") =>
                entry::from_rss_item(elem).ok(),
            ParserState::InFeed if elem.name.eq_ignore_ascii_case("entry") =>
                entry::from_atom_entry(elem).ok(),
            _ => None,
        }
    }
}

impl<R: Read> Iterator for FeedParser<R> {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        while let Some(event) = self.next_event() {
            // println!("{:?}", event);

            if let Ok(ref event) = event {
                if self.intercept_event(event) { continue }
            }

            match self.builder.handle_event(event) {
                Some(Ok(elem)) => {
                    if let Some(entry) = self.entry_from_element(elem) {
                        return Some(entry)
                    }
                }
                Some(Err(_)) => return None,
                None => (),
            }
        }

        None
    }
}

pub enum FeedParseError {
    Rss(ReadError),
    Atom(&'static str),
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, UTC};
    use super::FeedParser;

    static RSS_STR: &'static str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>TechCrunch</title>
    <link>http://techcrunch.com</link>
    <description>The latest technology news and information on startups</description>
    <item>
      <title>Ford hires Elon Musk as CEO</title>
      <pubDate>01 Apr 2019 07:30:00 GMT</pubDate>
      <description>In an unprecedented move, Ford hires Elon Musk.</description>
    </item>
  </channel>
</rss>
"#;

    static ATOM_STR: &'static str = r#"
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <id>urn:uuid:b3420f84-6bdf-4f46-a225-f1b9a14703b6</id>
  <title>TechCrunch</title>
  <updated>2019-04-01T07:30:00Z</updated>
  <entry>
    <id>urn:uuid:4ae8550b-2987-49fa-9f8c-54c180c418ac</id>
    <title>Ford hires Elon Musk as CEO</title>
    <updated>2019-04-01T07:30:00Z</updated>
  </entry>
</feed>
"#;

    #[test]
    fn test_rss_stream() {
        let mut parser = FeedParser::new(RSS_STR.as_bytes());

        let entry = parser.next().unwrap();
        assert_eq!(entry.title(), "Ford hires Elon Musk as CEO");
        assert_eq!(entry.content().unwrap(), "In an unprecedented move, Ford hires Elon Musk.");
        let expected_date = UTC.ymd(2019, 4, 1).and_hms(7, 30, 0);
        assert_eq!(entry.published().unwrap(), expected_date);

        assert!(parser.next().is_none());
    }

    #[test]
    fn test_atom_stream() {
        let mut parser = FeedParser::new(ATOM_STR.as_bytes());

        let entry = parser.next().unwrap();
        assert_eq!(entry.title(), "Ford hires Elon Musk as CEO");
        assert_eq!(entry.id().unwrap(), "urn:uuid:4ae8550b-2987-49fa-9f8c-54c180c418ac");
        let expected_date = UTC.ymd(2019, 4, 1).and_hms(7, 30, 0);
        assert_eq!(entry.published().unwrap(), expected_date);

        assert!(parser.next().is_none());
    }
}
