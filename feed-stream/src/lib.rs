extern crate xml;

mod str_buf_reader;

use std::ascii::AsciiExt;
use std::io::Read;

use xml::{Element, ElementBuilder, EndTag, Event, Parser, ParserError, StartTag};

use str_buf_reader::StrBufReader;

#[derive(Clone, Copy)]
enum ParserState {
    None,
    InRss,
    InChannel,
    InFeed,
}

pub struct RssParser<R> {
    reader: StrBufReader<R>,
    parser: Parser,
    builder: ElementBuilder,
    state: ParserState,
}

impl<R: Read> RssParser<R> {
    pub fn new(source: R) -> RssParser<R> {
        RssParser {
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
}

impl<R: Read> Iterator for RssParser<R> {
    type Item = Element;

    fn next(&mut self) -> Option<Element> {
        while let Some(event) = self.next_event() {
            // println!("{:?}", event);

            if let Ok(ref event) = event {
                if self.intercept_event(event) { continue }
            }

            match self.builder.handle_event(event) {
                Some(Ok(elem)) => {
                    if elem.name.eq_ignore_ascii_case("item") {
                        return Some(elem)
                    }
                }
                Some(Err(_)) => return None,
                None => (),
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::RssParser;

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

    #[test]
    fn test_rss_stream() {
        let mut parser = RssParser::new(RSS_STR.as_bytes());

        let elem = parser.next().unwrap();
        assert_eq!(elem.name, "item");
        assert_eq!(elem.get_child("title", None).unwrap().content_str(), "Ford hires Elon Musk as CEO");

        assert!(parser.next().is_none());
    }
}
