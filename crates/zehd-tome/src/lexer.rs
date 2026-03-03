use crate::cursor::Cursor;
use crate::error::{LexError, LexErrorKind};
use crate::token::{keyword_from_str, Span, Token, TokenKind};

// ── Mode stack for string interpolation ──────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum LexerMode {
    Normal,
    InterpolatedString,
    InterpolatedExpr { brace_depth: u32 },
}

// ── Lexer ────────────────────────────────────────────────────────────

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    tokens: Vec<Token>,
    errors: Vec<LexError>,
    mode_stack: Vec<LexerMode>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            cursor: Cursor::new(source),
            tokens: Vec::new(),
            errors: Vec::new(),
            mode_stack: vec![LexerMode::Normal],
        }
    }

    pub fn lex(mut self) -> (Vec<Token>, Vec<LexError>) {
        while !self.cursor.is_eof() {
            match self.current_mode() {
                LexerMode::Normal => self.scan_normal(),
                LexerMode::InterpolatedString => self.scan_interpolated_string(),
                LexerMode::InterpolatedExpr { .. } => self.scan_interpolated_expr(),
            }
        }

        // Report errors for any unclosed interpolated strings/exprs at EOF
        let pos = self.cursor.pos();
        for mode in self.mode_stack.drain(..).rev() {
            match mode {
                LexerMode::InterpolatedString | LexerMode::InterpolatedExpr { .. } => {
                    self.errors.push(LexError::new(
                        LexErrorKind::UnterminatedString,
                        Span::new(pos, pos),
                    ));
                }
                LexerMode::Normal => {}
            }
        }

        self.tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.cursor.pos(), self.cursor.pos()),
        ));
        (self.tokens, self.errors)
    }

    fn current_mode(&self) -> LexerMode {
        self.mode_stack.last().cloned().unwrap_or(LexerMode::Normal)
    }

    // ── Normal mode scanning ─────────────────────────────────────

    fn scan_normal(&mut self) {
        self.skip_whitespace();
        if self.cursor.is_eof() {
            return;
        }

        let start = self.cursor.pos();
        let byte = match self.cursor.peek() {
            Some(b) => b,
            Option::None => return,
        };

        match byte {
            // Comments or slash
            b'/' => {
                if self.cursor.peek_at(1) == Some(b'/') {
                    self.scan_line_comment();
                } else if self.cursor.peek_at(1) == Some(b'*') {
                    self.scan_block_comment();
                } else {
                    self.cursor.advance();
                    self.push(TokenKind::Slash, start);
                }
            }

            // String interpolation: $"
            b'$' if self.cursor.peek_at(1) == Some(b'"') => {
                self.cursor.advance(); // $
                self.cursor.advance(); // "
                self.push(TokenKind::InterpolatedStringStart, start);
                self.mode_stack.push(LexerMode::InterpolatedString);
            }

            // Regular string
            b'"' => self.scan_string(),

            // Numbers
            b'0'..=b'9' => self.scan_number(),

            // Identifiers and keywords
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.scan_identifier_or_keyword(),

            // Two-char operators and single-char fallbacks
            b'=' => {
                self.cursor.advance();
                if self.cursor.eat(b'=') {
                    self.push(TokenKind::EqEq, start);
                } else if self.cursor.eat(b'>') {
                    self.push(TokenKind::FatArrow, start);
                } else {
                    self.push(TokenKind::Eq, start);
                }
            }
            b'!' => {
                self.cursor.advance();
                if self.cursor.eat(b'=') {
                    self.push(TokenKind::BangEq, start);
                } else {
                    self.push(TokenKind::Bang, start);
                }
            }
            b'<' => {
                self.cursor.advance();
                if self.cursor.eat(b'=') {
                    self.push(TokenKind::LtEq, start);
                } else {
                    self.push(TokenKind::Lt, start);
                }
            }
            b'>' => {
                self.cursor.advance();
                if self.cursor.eat(b'=') {
                    self.push(TokenKind::GtEq, start);
                } else {
                    self.push(TokenKind::Gt, start);
                }
            }
            b'&' => {
                self.cursor.advance();
                if self.cursor.eat(b'&') {
                    self.push(TokenKind::AmpAmp, start);
                } else {
                    // Single & is not a valid token — report error, don't emit
                    self.errors.push(LexError::new(
                        LexErrorKind::UnexpectedCharacter('&'),
                        Span::new(start, self.cursor.pos()),
                    ));
                }
            }
            b'|' => {
                self.cursor.advance();
                if self.cursor.eat(b'|') {
                    self.push(TokenKind::PipePipe, start);
                } else {
                    self.errors.push(LexError::new(
                        LexErrorKind::UnexpectedCharacter('|'),
                        Span::new(start, self.cursor.pos()),
                    ));
                }
            }
            b':' => {
                self.cursor.advance();
                if self.cursor.eat(b':') {
                    self.push(TokenKind::ColonColon, start);
                } else {
                    self.push(TokenKind::Colon, start);
                }
            }
            b'.' => {
                self.cursor.advance();
                if self.cursor.peek() == Some(b'.') && self.cursor.peek_at(1) == Some(b'.') {
                    self.cursor.advance(); // second .
                    self.cursor.advance(); // third .
                    self.push(TokenKind::DotDotDot, start);
                } else {
                    self.push(TokenKind::Dot, start);
                }
            }

            // Single-char tokens
            b'+' => {
                self.cursor.advance();
                self.push(TokenKind::Plus, start);
            }
            b'-' => {
                self.cursor.advance();
                self.push(TokenKind::Minus, start);
            }
            b'*' => {
                self.cursor.advance();
                self.push(TokenKind::Star, start);
            }
            b'%' => {
                self.cursor.advance();
                self.push(TokenKind::Percent, start);
            }
            b'?' => {
                self.cursor.advance();
                self.push(TokenKind::Question, start);
            }
            b'#' => {
                self.cursor.advance();
                self.push(TokenKind::Hash, start);
            }
            b'{' => {
                self.cursor.advance();
                self.push(TokenKind::LeftBrace, start);
            }
            b'}' => {
                self.cursor.advance();
                self.push(TokenKind::RightBrace, start);
            }
            b'(' => {
                self.cursor.advance();
                self.push(TokenKind::LeftParen, start);
            }
            b')' => {
                self.cursor.advance();
                self.push(TokenKind::RightParen, start);
            }
            b'[' => {
                self.cursor.advance();
                self.push(TokenKind::LeftBracket, start);
            }
            b']' => {
                self.cursor.advance();
                self.push(TokenKind::RightBracket, start);
            }
            b';' => {
                self.cursor.advance();
                self.push(TokenKind::Semicolon, start);
            }
            b',' => {
                self.cursor.advance();
                self.push(TokenKind::Comma, start);
            }

            _ => {
                self.cursor.advance();
                self.errors.push(LexError::new(
                    LexErrorKind::UnexpectedCharacter(byte as char),
                    Span::new(start, self.cursor.pos()),
                ));
            }
        }
    }

    // ── Interpolated string mode ─────────────────────────────────

    fn scan_interpolated_string(&mut self) {
        let start = self.cursor.pos();

        match self.cursor.peek() {
            // End of interpolated string
            Some(b'"') => {
                self.cursor.advance();
                self.push(TokenKind::InterpolatedStringEnd, start);
                self.mode_stack.pop();
            }
            // Expression interpolation start
            Some(b'{') => {
                self.cursor.advance();
                self.push(TokenKind::InterpolatedExprStart, start);
                self.mode_stack
                    .push(LexerMode::InterpolatedExpr { brace_depth: 1 });
            }
            // End of file inside string — error
            Option::None => {
                self.errors.push(LexError::new(
                    LexErrorKind::UnterminatedString,
                    Span::new(start, self.cursor.pos()),
                ));
                self.mode_stack.pop();
            }
            // String fragment — collect text until `{`, `"`, or EOF
            Some(_) => {
                self.scan_string_fragment(start);
            }
        }
    }

    fn scan_string_fragment(&mut self, start: u32) {
        loop {
            match self.cursor.peek() {
                Some(b'"') | Some(b'{') | Option::None => break,
                Some(b'\\') => {
                    // Consume escape sequence
                    self.cursor.advance(); // backslash
                    if self.cursor.peek().is_some() {
                        self.cursor.advance(); // escaped char
                    }
                }
                Some(_) => {
                    self.cursor.advance();
                }
            }
        }
        if self.cursor.pos() > start {
            self.push(TokenKind::StringFragment, start);
        }
    }

    // ── Interpolated expression mode ─────────────────────────────

    fn scan_interpolated_expr(&mut self) {
        self.skip_whitespace();
        if self.cursor.is_eof() {
            return;
        }

        let start = self.cursor.pos();
        let byte = match self.cursor.peek() {
            Some(b) => b,
            Option::None => return,
        };

        // Track brace depth for nested braces inside expressions
        match byte {
            b'{' => {
                self.cursor.advance();
                // Increase brace depth
                if let Some(LexerMode::InterpolatedExpr { brace_depth }) =
                    self.mode_stack.last_mut()
                {
                    *brace_depth += 1;
                }
                self.push(TokenKind::LeftBrace, start);
            }
            b'}' => {
                let should_pop = matches!(
                    self.mode_stack.last(),
                    Some(LexerMode::InterpolatedExpr { brace_depth: 1 })
                );

                if should_pop {
                    self.cursor.advance();
                    self.push(TokenKind::InterpolatedExprEnd, start);
                    self.mode_stack.pop();
                } else {
                    // Nested brace — decrease depth
                    self.cursor.advance();
                    if let Some(LexerMode::InterpolatedExpr { brace_depth }) =
                        self.mode_stack.last_mut()
                    {
                        *brace_depth -= 1;
                    }
                    self.push(TokenKind::RightBrace, start);
                }
            }
            // Nested interpolated string inside expression
            b'$' if self.cursor.peek_at(1) == Some(b'"') => {
                self.cursor.advance(); // $
                self.cursor.advance(); // "
                self.push(TokenKind::InterpolatedStringStart, start);
                self.mode_stack.push(LexerMode::InterpolatedString);
            }
            // Everything else: delegate to normal scanning
            _ => self.scan_normal(),
        }
    }

    // ── Scanning helpers ─────────────────────────────────────────

    fn skip_whitespace(&mut self) {
        self.cursor
            .eat_while(|b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r');
    }

    fn scan_line_comment(&mut self) {
        // Consume // and the rest of the line
        self.cursor.advance(); // first /
        self.cursor.advance(); // second /
        self.cursor.eat_while(|b| b != b'\n');
    }

    fn scan_block_comment(&mut self) {
        let start = self.cursor.pos();
        self.cursor.advance(); // /
        self.cursor.advance(); // *

        loop {
            match self.cursor.peek() {
                Option::None => {
                    self.errors.push(LexError::new(
                        LexErrorKind::UnterminatedBlockComment,
                        Span::new(start, self.cursor.pos()),
                    ));
                    return;
                }
                Some(b'*') if self.cursor.peek_at(1) == Some(b'/') => {
                    self.cursor.advance(); // *
                    self.cursor.advance(); // /
                    return;
                }
                Some(_) => {
                    self.cursor.advance();
                }
            }
        }
    }

    fn scan_string(&mut self) {
        let start = self.cursor.pos();
        self.cursor.advance(); // opening "

        loop {
            match self.cursor.peek() {
                Option::None | Some(b'\n') => {
                    self.errors.push(LexError::new(
                        LexErrorKind::UnterminatedString,
                        Span::new(start, self.cursor.pos()),
                    ));
                    return;
                }
                Some(b'"') => {
                    self.cursor.advance(); // closing "
                    self.push(TokenKind::String, start);
                    return;
                }
                Some(b'\\') => {
                    self.cursor.advance(); // backslash
                    match self.cursor.peek() {
                        Some(b'n' | b't' | b'r' | b'\\' | b'"' | b'0') => {
                            self.cursor.advance();
                        }
                        Some(c) => {
                            let esc_start = self.cursor.pos() - 1; // backslash pos
                            self.cursor.advance();
                            self.errors.push(LexError::new(
                                LexErrorKind::InvalidEscapeSequence(c as char),
                                Span::new(esc_start, self.cursor.pos()),
                            ));
                        }
                        Option::None => {
                            self.errors.push(LexError::new(
                                LexErrorKind::UnterminatedString,
                                Span::new(start, self.cursor.pos()),
                            ));
                            return;
                        }
                    }
                }
                Some(_) => {
                    self.cursor.advance();
                }
            }
        }
    }

    fn scan_number(&mut self) {
        let start = self.cursor.pos();

        // Consume digits
        self.cursor.eat_while(|b| b.is_ascii_digit());

        // Check for float
        if self.cursor.peek() == Some(b'.')
            && self.cursor.peek_at(1).is_some_and(|b| b.is_ascii_digit())
        {
            self.cursor.advance(); // .
            self.cursor.eat_while(|b| b.is_ascii_digit());
            let text = self.cursor.slice(start as usize, self.cursor.pos() as usize);
            match text.parse::<f64>() {
                Ok(v) => self.push(TokenKind::Float(v), start),
                Result::Err(_) => {
                    self.errors.push(LexError::new(
                        LexErrorKind::NumberOverflow,
                        Span::new(start, self.cursor.pos()),
                    ));
                }
            }
            return;
        }

        // Check for time suffix or invalid suffix
        let num_end = self.cursor.pos();
        let num_text = self.cursor.slice(start as usize, num_end as usize);

        if let Some(b) = self.cursor.peek() {
            if b.is_ascii_alphabetic() || b == b'_' {
                // Potential suffix — read it all
                let suffix_start = self.cursor.pos();
                self.cursor.eat_while(|b| b.is_ascii_alphanumeric() || b == b'_');
                let suffix = self
                    .cursor
                    .slice(suffix_start as usize, self.cursor.pos() as usize);

                match suffix {
                    "ms" | "s" | "m" | "h" => {
                        let base: u64 = match num_text.parse() {
                            Ok(v) => v,
                            Result::Err(_) => {
                                self.errors.push(LexError::new(
                                    LexErrorKind::NumberOverflow,
                                    Span::new(start, self.cursor.pos()),
                                ));
                                return;
                            }
                        };
                        let multiplier: u64 = match suffix {
                            "ms" => 1,
                            "s" => 1000,
                            "m" => 60_000,
                            "h" => 3_600_000,
                            _ => unreachable!(),
                        };
                        match base.checked_mul(multiplier) {
                            Some(ms) => self.push(TokenKind::TimeLiteral(ms), start),
                            Option::None => {
                                self.errors.push(LexError::new(
                                    LexErrorKind::NumberOverflow,
                                    Span::new(start, self.cursor.pos()),
                                ));
                            }
                        }
                    }
                    _ => {
                        self.errors.push(LexError::new(
                            LexErrorKind::InvalidNumberSuffix(suffix.to_string()),
                            Span::new(start, self.cursor.pos()),
                        ));
                    }
                }
                return;
            }
        }

        // Plain integer
        match num_text.parse::<i64>() {
            Ok(v) => self.push(TokenKind::Integer(v), start),
            Result::Err(_) => {
                self.errors.push(LexError::new(
                    LexErrorKind::NumberOverflow,
                    Span::new(start, self.cursor.pos()),
                ));
            }
        }
    }

    fn scan_identifier_or_keyword(&mut self) {
        let start = self.cursor.pos();

        // First char already matched [a-zA-Z_]
        self.cursor.advance();
        self.cursor
            .eat_while(|b| b.is_ascii_alphanumeric() || b == b'_');

        let text = self.cursor.slice(start as usize, self.cursor.pos() as usize);

        // Single underscore is a special token
        if text == "_" {
            self.push(TokenKind::Underscore, start);
            return;
        }

        let kind = keyword_from_str(text).unwrap_or(TokenKind::Identifier);
        self.push(kind, start);
    }

    // ── Token emission ───────────────────────────────────────────

    fn push(&mut self, kind: TokenKind, start: u32) {
        self.tokens.push(Token::new(
            kind,
            Span::new(start, self.cursor.pos()),
        ));
    }
}
