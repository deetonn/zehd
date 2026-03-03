/// Byte-offset span into source text. Compact (8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Length in bytes.
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Extract the lexeme from the original source.
    pub fn lexeme<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start as usize..self.end as usize]
    }
}

/// A single token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Extract the lexeme from the original source.
    pub fn lexeme<'a>(&self, source: &'a str) -> &'a str {
        self.span.lexeme(source)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Keywords ──────────────────────────────────────────────
    Let,
    Const,
    Fn,
    If,
    Else,
    Match,
    For,
    In,
    While,
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Init,
    Error,
    Type,
    Enum,
    Import,
    From,
    Return,
    Break,
    Continue,
    SelfKw,
    True,
    False,
    None,
    Some,
    Ok,
    Err,

    // ── Literals ──────────────────────────────────────────────
    Integer(i64),
    Float(f64),
    String,
    /// Time literal — value stored in milliseconds.
    TimeLiteral(u64),

    // ── Interpolated string tokens ────────────────────────────
    /// The opening `$"` of an interpolated string.
    InterpolatedStringStart,
    /// A literal fragment inside an interpolated string.
    StringFragment,
    /// The `{` that opens an expression inside an interpolated string.
    InterpolatedExprStart,
    /// The `}` that closes an expression inside an interpolated string.
    InterpolatedExprEnd,
    /// The closing `"` of an interpolated string.
    InterpolatedStringEnd,

    // ── Operators ─────────────────────────────────────────────
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    EqEq,       // ==
    BangEq,     // !=
    Lt,         // <
    Gt,         // >
    LtEq,       // <=
    GtEq,       // >=
    AmpAmp,     // &&
    PipePipe,   // ||
    Bang,       // !
    Eq,         // =
    Question,   // ?
    FatArrow,   // =>
    Dot,        // .
    ColonColon, // ::

    // ── Delimiters ────────────────────────────────────────────
    LeftBrace,    // {
    RightBrace,   // }
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    Semicolon,    // ;
    Comma,        // ,
    Colon,        // :

    // ── Special ───────────────────────────────────────────────
    Hash,       // #
    Underscore, // _
    DotDotDot,  // ...
    Identifier,

    Eof,
}

impl TokenKind {
    /// Returns `true` if this is a keyword token.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Let
                | TokenKind::Const
                | TokenKind::Fn
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::Match
                | TokenKind::For
                | TokenKind::In
                | TokenKind::While
                | TokenKind::Get
                | TokenKind::Post
                | TokenKind::Put
                | TokenKind::Patch
                | TokenKind::Delete
                | TokenKind::Init
                | TokenKind::Error
                | TokenKind::Type
                | TokenKind::Enum
                | TokenKind::Import
                | TokenKind::From
                | TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::SelfKw
                | TokenKind::True
                | TokenKind::False
                | TokenKind::None
                | TokenKind::Some
                | TokenKind::Ok
                | TokenKind::Err
        )
    }
}

/// Look up a keyword from its text. Returns `None` for identifiers.
pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
    match s {
        "let" => Some(TokenKind::Let),
        "const" => Some(TokenKind::Const),
        "fn" => Some(TokenKind::Fn),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "match" => Some(TokenKind::Match),
        "for" => Some(TokenKind::For),
        "in" => Some(TokenKind::In),
        "while" => Some(TokenKind::While),
        "get" => Some(TokenKind::Get),
        "post" => Some(TokenKind::Post),
        "put" => Some(TokenKind::Put),
        "patch" => Some(TokenKind::Patch),
        "delete" => Some(TokenKind::Delete),
        "init" => Some(TokenKind::Init),
        "error" => Some(TokenKind::Error),
        "type" => Some(TokenKind::Type),
        "enum" => Some(TokenKind::Enum),
        "import" => Some(TokenKind::Import),
        "from" => Some(TokenKind::From),
        "return" => Some(TokenKind::Return),
        "break" => Some(TokenKind::Break),
        "continue" => Some(TokenKind::Continue),
        "self" => Some(TokenKind::SelfKw),
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),
        "None" => Some(TokenKind::None),
        "Some" => Some(TokenKind::Some),
        "Ok" => Some(TokenKind::Ok),
        "Err" => Some(TokenKind::Err),
        _ => Option::None,
    }
}
