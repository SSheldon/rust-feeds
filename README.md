# rust-atom

Rust library for serializing the Atom web content syndication format.

 - [Documentation](https://docs.rs/atom_syndication)
 - [Crate](https://crates.io/crates/atom_syndication)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
atom_syndication = "0.3"
```

and this to your crate root:

```rust
extern crate atom_syndication;
```

## Examples

### Writing

```rust
use atom::{Feed, Entry};

let entry = Entry {
    id: String::from("urn:uuid:4ae8550b-2987-49fa-9f8c-54c180c418ac"),
    title: String::from("Ford hires Elon Musk as CEO"),
    updated: String::from("2019-04-01T07:30:00Z"),
    ..Default::default()
};

let feed = Feed {
    id: String::from("urn:uuid:b3420f84-6bdf-4f46-a225-f1b9a14703b6"),
    title: String::from("TechCrunch"),
    updated: String::from("2019-04-01T07:30:00Z"),
    entries: vec![entry],
    ..Default::default()
};

let atom_string = feed.to_string();
```

### Reading

```rust
use atom::Feed;

let atom_str = r#"
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

let feed = atom_str.parse::<Feed>().unwrap();
```

## Acknowledgements

Thanks to:

 - Francis Gagné for contributing many improvements to the quality of
   this library, including writing an extensive test suite.
 - Corey Farwell for writing [rust-rss](https://github.com/frewsxcv/rust-rss).
   This library is a fairly direct port of it to Atom.

## License

Licensed under either of

 - Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 - MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
