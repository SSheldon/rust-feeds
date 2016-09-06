use std::io::Read;
use std::str;

use xml::{Element, ElementBuilder, Event, Parser, ParserError, StartTag};

struct StrBufReader<R> {
    reader: R,
    buffer: Vec<u8>,
    last: usize,
}

impl<R: Read> StrBufReader<R> {
    fn next_str(&mut self) -> Option<&str> {
        // copy extra bytes to the front
        let extra_bytes = self.buffer.len() - self.last;
        for i in 0..extra_bytes {
            self.buffer[i] = self.buffer[self.last + i];
        }

        let capacity = self.buffer.capacity();
        self.buffer.resize(capacity, 0);
        let new_bytes = self.reader.read(&mut self.buffer[extra_bytes..]);
        self.buffer.truncate(extra_bytes + new_bytes.as_ref().ok().map_or(0, |&b| b));

        // find a character boundary
        self.last = match new_bytes {
            // If there are no more bytes coming, don't save any extra bytes
            Ok(0) => self.buffer.len(),
            _ => self.buffer.iter().rposition(|&b| b < 128 || b >= 192).unwrap_or(0),
        };

        if self.last > 0 {
            str::from_utf8(&self.buffer[..self.last]).ok()
        } else {
            None
        }
    }
}

pub struct RssParser {
    parser: Parser,
    builder: ElementBuilder,
}

impl RssParser {
    pub fn new(s: &str) -> Result<RssParser, ParserError> {
        let mut parser = Parser::new();
        parser.feed_str(&s);

        for event in &mut parser {
            let event = match event {
                Ok(o) => o,
                Err(e) => return Err(e),
            };

            match event {
                Event::ElementStart(StartTag { ref name, .. }) if name == "channel" => break,
                _ => (),
            }
        }

        Ok(RssParser {
            parser: parser,
            builder: ElementBuilder::new(),
        })
    }
}

impl Iterator for RssParser {
    type Item = Element;

    fn next(&mut self) -> Option<Element> {
        for event in &mut self.parser {
            // println!("{:?}", event);
            match self.builder.handle_event(event) {
                Some(Ok(elem)) => {
                    if elem.name == "item" {
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
