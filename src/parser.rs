use std::io::{ErrorKind, Read, self};
use std::str;

use xml::{Element, ElementBuilder, Event, Parser, ParserError, StartTag};

struct StrBufReader<R> {
    reader: R,
    buffer: Vec<u8>,
    len: usize,
    extra: usize,
}

impl<R: Read> StrBufReader<R> {
    fn with_capacity(capacity: usize, source: R) -> StrBufReader<R> {
        StrBufReader {
            reader: source,
            buffer: vec![0; capacity],
            len: 0,
            extra: 0,
        }
    }

    fn next_str(&mut self) -> Option<io::Result<&str>> {
        // copy extra bytes to the front
        for i in 0..self.extra {
            self.buffer[i] = self.buffer[self.len + i];
        }
        self.len = 0;

        let new_bytes = self.reader.read(&mut self.buffer[self.extra..]);
        // find a character boundary
        let (len, extra) = match new_bytes {
            // If there are no more bytes coming, don't save any extra bytes
            Ok(0) => (self.extra, 0),
            Ok(i) => {
                let full_len = self.extra + i;
                let last = (&self.buffer[..full_len]).iter()
                    .rposition(|&b| b < 128 || b >= 192)
                    .unwrap_or(0);
                (last, full_len - last)
            },
            Err(e) => return Some(Err(e)),
        };
        self.len = len;
        self.extra = extra;

        if len > 0 {
            Some(str::from_utf8(&self.buffer[..len]).map_err(|e| {
                io::Error::new(ErrorKind::InvalidData, e)
            }))
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
