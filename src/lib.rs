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
const XHTML_NS: &'static str = "http://www.w3.org/1999/xhtml";
