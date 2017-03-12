## 0.3.0

Breaking changes:

 - Changed type of `Entry::content` from `Option<String>` to `Option<Content>`.
   This allows content elements to be marked as plain text, text with escaped
   HTML, or an embedded XHTML document.
 - `Feed` and `Entry` no longer derive `Eq`, only `PartialEq`.

## 0.2.0

Breaking changes:

 - Changed type of `Entry::source` from `Option<Feed>` to `Option<Source>`.
