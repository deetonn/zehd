/// A byte-oriented cursor over source text.
///
/// All positions are byte offsets into the original UTF-8 source string.
pub struct Cursor<'a> {
    source: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            pos: 0,
        }
    }

    /// Current byte offset.
    #[inline]
    pub fn pos(&self) -> u32 {
        self.pos as u32
    }

    /// Returns `true` when all input has been consumed.
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    /// Peek at the current byte without advancing.
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        self.source.get(self.pos).copied()
    }

    /// Peek at the byte `n` positions ahead without advancing.
    #[inline]
    pub fn peek_at(&self, n: usize) -> Option<u8> {
        self.source.get(self.pos + n).copied()
    }

    /// Advance one byte and return it.
    #[inline]
    pub fn advance(&mut self) -> Option<u8> {
        let byte = self.source.get(self.pos).copied()?;
        self.pos += 1;
        Some(byte)
    }

    /// Advance if the current byte matches `expected`.
    #[inline]
    pub fn eat(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Advance while the predicate holds. Returns the number of bytes consumed.
    pub fn eat_while<F: Fn(u8) -> bool>(&mut self, predicate: F) -> usize {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if predicate(b) {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.pos - start
    }

    /// Extract a slice of the original source between byte offsets.
    #[inline]
    pub fn slice(&self, start: usize, end: usize) -> &'a str {
        // Safety: the source is valid UTF-8 and we only split on ASCII boundaries
        // (identifiers, numbers, operators are all ASCII; strings are tracked by byte offset).
        // If we ever need to handle non-ASCII identifiers, we'll need to be more careful here.
        std::str::from_utf8(&self.source[start..end])
            .expect("slice should be valid UTF-8")
    }
}
