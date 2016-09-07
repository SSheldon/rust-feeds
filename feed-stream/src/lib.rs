extern crate xml;

mod str_buf_reader;

use std::ascii::AsciiExt;
use std::io::Read;

use xml::{Element, ElementBuilder, Event, Parser, ParserError, StartTag};

use str_buf_reader::StrBufReader;

pub struct RssParser<R> {
    reader: StrBufReader<R>,
    parser: Parser,
    builder: ElementBuilder,
}

impl<R: Read> RssParser<R> {
    pub fn new(source: R) -> Result<RssParser<R>, ParserError> {
        let mut parser = RssParser {
            reader: StrBufReader::with_capacity(4096, source),
            parser: Parser::new(),
            builder: ElementBuilder::new(),
        };

        while let Some(event) = parser.next_event() {
            let event = try!(event);
            match event {
                Event::ElementStart(StartTag { ref name, .. }) if name.eq_ignore_ascii_case("channel") => break,
                _ => (),
            }
        }

        Ok(parser)
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
