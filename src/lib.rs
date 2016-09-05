// Copyright 2015 Corey Farwell
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Library for serializing the Atom web content syndication format
//!
//! # Examples
//!
//! ## Writing
//!
//! ```
//! use atom_syndication::{Feed, Entry};
//!
//! let entry = Entry {
//!     id: String::from("urn:uuid:4ae8550b-2987-49fa-9f8c-54c180c418ac"),
//!     title: String::from("Ford hires Elon Musk as CEO"),
//!     updated: String::from("2019-04-01T07:30:00Z"),
//!     ..Default::default()
//! };
//!
//! let feed = Feed {
//!     id: String::from("urn:uuid:b3420f84-6bdf-4f46-a225-f1b9a14703b6"),
//!     title: String::from("TechCrunch"),
//!     updated: String::from("2019-04-01T07:30:00Z"),
//!     entries: vec![entry],
//!     ..Default::default()
//! };
//!
//! let atom_string = feed.to_string();
//! ```
//!
//! ## Reading
//!
//! ```
//! use atom_syndication::Feed;
//!
//! let atom_str = r#"
//! <?xml version="1.0" encoding="utf-8"?>
//! <feed xmlns="http://www.w3.org/2005/Atom">
//!   <id>urn:uuid:b3420f84-6bdf-4f46-a225-f1b9a14703b6</id>
//!   <title>TechCrunch</title>
//!   <updated>2019-04-01T07:30:00Z</updated>
//!   <entry>
//!     <id>urn:uuid:4ae8550b-2987-49fa-9f8c-54c180c418ac</id>
//!     <title>Ford hires Elon Musk as CEO</title>
//!     <updated>2019-04-01T07:30:00Z</updated>
//!   </entry>
//! </feed>
//! "#;
//!
//! let feed = atom_str.parse::<Feed>().unwrap();
//! ```

extern crate xml;

mod author;
mod category;
mod contributor;
mod entry;
mod feed;
mod generator;
mod link;
mod person;
mod source;
mod utils;

pub use category::Category;
pub use entry::Entry;
pub use feed::Feed;
pub use generator::Generator;
pub use link::Link;
pub use person::Person;
pub use source::Source;


const NS: &'static str = "http://www.w3.org/2005/Atom";


#[cfg(test)]
mod test {
    use std::default::Default;
    use std::fs::File;
    use std::io::Read;
    use std::str::FromStr;
    use super::{Person, Entry, Feed, Link, Source};

    #[test]
    fn test_basic_to_string() {
        let author = Person {
            name: "N. Blogger".to_string(),
            ..Default::default()
        };

        let entry = Entry {
            title: "My first post!".to_string(),
            content: Some("This is my first post".to_string()),
            ..Default::default()
        };

        let feed = Feed {
            title: "My Blog".to_string(),
            authors: vec![author],
            entries: vec![entry],
            ..Default::default()
        };

        assert_eq!(feed.to_string(), "<?xml version=\"1.0\" encoding=\"utf-8\"?><feed xmlns=\'http://www.w3.org/2005/Atom\'><id></id><title>My Blog</title><updated></updated><author><name>N. Blogger</name></author><entry><id></id><title>My first post!</title><updated></updated><content>This is my first post</content></entry></feed>");
    }

    #[test]
    fn test_links() {
        let feed = Feed {
            links: vec![
                Link {
                    href: "http://test.blog/blog.atom".to_string(),
                    rel: Some("self".to_string()),
                    ..Default::default()
                },
            ],
            entries: vec![
                Entry {
                    links: vec![
                        Link {
                            href: "http://test.blog/entry".to_string(),
                            rel: Some("alternate".to_string()),
                            ..Default::default()
                        }
                    ],
                    source: Some(Source {
                        title: Some("Original Blog".to_string()),
                        links: vec![
                            Link {
                                href: "http://original.blog/feed.atom".to_string(),
                                rel: Some("self".to_string()),
                                ..Default::default()
                            }
                        ],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert!(feed.to_string().bytes().count() > 0);
    }

    #[test]
    fn test_from_file() {
        let mut file = File::open("test-data/xkcd.xml").unwrap();
        let mut atom_string = String::new();
        file.read_to_string(&mut atom_string).unwrap();
        let feed = Feed::from_str(&atom_string).unwrap();
        assert!(feed.to_string().len() > 0);
    }

    #[test]
    fn test_read_no_feeds() {
        let atom_str = "";
        assert!(Feed::from_str(atom_str).is_err());
    }

    #[test]
    fn test_read_one_feed_no_properties() {
        let atom_str = "\
            <feed>\
            </feed>";
        assert!(Feed::from_str(atom_str).is_err());
    }

    #[test]
    fn test_read_one_feed() {
        let atom_str = r#"
            <feed xmlns="http://www.w3.org/2005/Atom">
                <id></id>
                <title>Hello world!</title>
                <updated></updated>
                <description></description>
            </feed>"#;
        println!("{}", atom_str);
        let feed = Feed::from_str(atom_str).unwrap();
        assert_eq!("Hello world!", feed.title);
    }

    // Ensure reader ignores the PI XML node and continues to parse the feed
    #[test]
    fn test_read_with_pinode() {
        let atom_str = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom">
                <id></id>
                <title>Title</title>
                <updated></updated>
                <description></description>
            </feed>"#;
        let feed = Feed::from_str(atom_str).unwrap();
        assert_eq!("Title", feed.title);
    }
}
