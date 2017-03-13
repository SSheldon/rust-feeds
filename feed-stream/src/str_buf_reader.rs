use std::io::{ErrorKind, Read, self};
use std::str;

pub struct StrBufReader<R> {
    reader: R,
    buffer: Vec<u8>,
    len: usize,
    extra: usize,
}

impl<R: Read> StrBufReader<R> {
    pub fn with_capacity(capacity: usize, source: R) -> StrBufReader<R> {
        StrBufReader {
            reader: source,
            buffer: vec![0; capacity],
            len: 0,
            extra: 0,
        }
    }

    pub fn next_str(&mut self) -> io::Result<Option<&str>> {
        // copy extra bytes to the front
        for i in 0..self.extra {
            self.buffer[i] = self.buffer[self.len + i];
        }
        self.len = 0;

        let new_bytes = self.reader.read(&mut self.buffer[self.extra..])?;
        // find a character boundary
        let (len, extra) = match new_bytes {
            // If there are no more bytes coming, don't save any extra bytes
            0 => (self.extra, 0),
            i => {
                let full_len = self.extra + i;
                let last = (&self.buffer[..full_len]).iter()
                    .rposition(|&b| b < 128 || b >= 192)
                    .unwrap_or(0);
                (last, full_len - last)
            },
        };
        self.len = len;
        self.extra = extra;

        if len == 0 && extra == 0 {
            Ok(None)
        } else {
            str::from_utf8(&self.buffer[..len])
                .map(|s| Some(s))
                .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use super::StrBufReader;

    fn read_to_end<R: io::Read>(mut reader: StrBufReader<R>)
            -> io::Result<String> {
        let mut result = String::new();
        while let Some(s) = reader.next_str()? {
            result.push_str(s);
        }
        Ok(result)
    }

    #[test]
    fn test_fits() {
        let data = "foo";
        let reader = StrBufReader::with_capacity(64, data.as_bytes());
        assert_eq!(read_to_end(reader).unwrap(), data);
    }

    #[test]
    fn test_split() {
        let data = "foobarbaz";
        let reader = StrBufReader::with_capacity(4, data.as_bytes());
        assert_eq!(read_to_end(reader).unwrap(), data);
    }

    #[test]
    fn test_unicode_fits() {
        let data = "💖💖💖💖";
        let reader = StrBufReader::with_capacity(64, data.as_bytes());
        assert_eq!(read_to_end(reader).unwrap(), data);
    }

    #[test]
    fn test_unicode_split() {
        let data = "💖💖💖💖";
        let reader = StrBufReader::with_capacity(12, data.as_bytes());
        assert_eq!(read_to_end(reader).unwrap(), data);
    }

    #[test]
    fn test_last_buf_single_wide_char() {
        let data = "fooooooooooooo💖";
        let reader = StrBufReader::with_capacity(16, data.as_bytes());
        assert_eq!(read_to_end(reader).unwrap(), data);
    }

    #[test]
    fn test_invalid_bytes() {
        let data = b"fo\xF0\x9F\x92\x96ooo\x00\x9F\x92\x96o";
        let mut reader = StrBufReader::with_capacity(8, data.as_ref());
        assert_eq!(reader.next_str().unwrap().unwrap(), "fo💖o");
        let err = reader.next_str().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
