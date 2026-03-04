use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString,
    Position, Range, Url,
};
use zehd_codex::error::Severity as ParseSeverity;
use zehd_sigil::error::Severity as TypeSeverity;
use zehd_tome::token::Span;

/// Maps byte offsets in source text to LSP line/column positions.
pub struct LineIndex {
    /// Byte offset of the start of each line.
    line_starts: Vec<u32>,
}

impl LineIndex {
    /// Build a line index by scanning for newlines. O(n) in source length.
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0u32];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        Self { line_starts }
    }

    /// Convert a byte offset to an LSP Position (0-based line, UTF-16 column).
    pub fn offset_to_position(&self, offset: u32, source: &str) -> Position {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(next) => next - 1,
        };
        let line_start = self.line_starts[line] as usize;
        let offset = offset as usize;
        // Clamp to source length to handle spans at EOF.
        let offset = offset.min(source.len());
        let col_utf16: u32 = source[line_start..offset]
            .chars()
            .map(|c| c.len_utf16() as u32)
            .sum();
        Position::new(line as u32, col_utf16)
    }

    /// Convert a Span to an LSP Range.
    pub fn span_to_range(&self, span: Span, source: &str) -> Range {
        Range::new(
            self.offset_to_position(span.start, source),
            self.offset_to_position(span.end, source),
        )
    }
}

/// Run parse + type-check on `source` and return LSP diagnostics.
pub fn compute(uri: &Url, source: &str) -> Vec<Diagnostic> {
    let line_index = LineIndex::new(source);
    let mut diagnostics = Vec::new();

    let parse_result = zehd_codex::parse(source);

    for err in &parse_result.errors {
        let severity = match err.severity {
            ParseSeverity::Error => DiagnosticSeverity::ERROR,
            ParseSeverity::Warning => DiagnosticSeverity::WARNING,
        };

        let mut message = err.message.clone();
        for note in &err.notes {
            message.push_str("\nnote: ");
            message.push_str(note);
        }

        let related_information = if err.labels.is_empty() {
            None
        } else {
            Some(
                err.labels
                    .iter()
                    .map(|label| DiagnosticRelatedInformation {
                        location: Location {
                            uri: uri.clone(),
                            range: line_index.span_to_range(label.span, source),
                        },
                        message: label.message.clone(),
                    })
                    .collect(),
            )
        };

        diagnostics.push(Diagnostic {
            range: line_index.span_to_range(err.primary_span, source),
            severity: Some(severity),
            code: Some(NumberOrString::String(err.code.to_string())),
            source: Some("zehd".into()),
            message,
            related_information,
            ..Default::default()
        });
    }

    // Only run type-check if parse produced an AST (it always does, but may be partial).
    let check_result = zehd_sigil::check(&parse_result.program, source, &Default::default());

    for err in &check_result.errors {
        let severity = match err.severity {
            TypeSeverity::Error => DiagnosticSeverity::ERROR,
            TypeSeverity::Warning => DiagnosticSeverity::WARNING,
        };

        let mut message = err.message.clone();
        for note in &err.notes {
            message.push_str("\nnote: ");
            message.push_str(note);
        }

        let related_information = if err.labels.is_empty() {
            None
        } else {
            Some(
                err.labels
                    .iter()
                    .map(|label| DiagnosticRelatedInformation {
                        location: Location {
                            uri: uri.clone(),
                            range: line_index.span_to_range(label.span, source),
                        },
                        message: label.message.clone(),
                    })
                    .collect(),
            )
        };

        diagnostics.push(Diagnostic {
            range: line_index.span_to_range(err.primary_span, source),
            severity: Some(severity),
            code: Some(NumberOrString::String(err.code.to_string())),
            source: Some("zehd".into()),
            message,
            related_information,
            ..Default::default()
        });
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_index_single_line() {
        let source = "let x = 42;";
        let idx = LineIndex::new(source);
        assert_eq!(idx.offset_to_position(0, source), Position::new(0, 0));
        assert_eq!(idx.offset_to_position(4, source), Position::new(0, 4));
        assert_eq!(idx.offset_to_position(11, source), Position::new(0, 11));
    }

    #[test]
    fn line_index_multi_line() {
        let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
        let idx = LineIndex::new(source);
        // Line 0
        assert_eq!(idx.offset_to_position(0, source), Position::new(0, 0));
        // Line 1 starts at byte 11
        assert_eq!(idx.offset_to_position(11, source), Position::new(1, 0));
        assert_eq!(idx.offset_to_position(15, source), Position::new(1, 4));
        // Line 2 starts at byte 22
        assert_eq!(idx.offset_to_position(22, source), Position::new(2, 0));
    }

    #[test]
    fn line_index_empty_source() {
        let source = "";
        let idx = LineIndex::new(source);
        assert_eq!(idx.offset_to_position(0, source), Position::new(0, 0));
    }

    #[test]
    fn line_index_span_to_range() {
        let source = "let x = 1;\nlet y = 2;";
        let idx = LineIndex::new(source);
        let span = Span::new(11, 14); // "let" on line 2
        let range = idx.span_to_range(span, source);
        assert_eq!(range.start, Position::new(1, 0));
        assert_eq!(range.end, Position::new(1, 3));
    }

    #[test]
    fn compute_returns_parse_errors() {
        let uri = Url::parse("file:///test.z").unwrap();
        let source = "let x = ;";
        let diagnostics = compute(&uri, source);
        assert!(!diagnostics.is_empty());
        // Should have source "zehd"
        assert_eq!(diagnostics[0].source.as_deref(), Some("zehd"));
        // Should have an error code starting with "E"
        if let Some(NumberOrString::String(code)) = &diagnostics[0].code {
            assert!(code.starts_with('E'), "expected E-prefixed code, got {code}");
        } else {
            panic!("expected string error code");
        }
    }

    #[test]
    fn compute_valid_source_no_parse_errors() {
        let uri = Url::parse("file:///test.z").unwrap();
        let source = "let x = 42;";
        let diagnostics = compute(&uri, source);
        // No parse errors for valid syntax — may still have type errors,
        // but should not have parse errors.
        let parse_errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| {
                matches!(
                    &d.code,
                    Some(NumberOrString::String(c)) if c.starts_with('E')
                )
            })
            .collect();
        assert!(parse_errors.is_empty(), "unexpected parse errors: {parse_errors:?}");
    }
}
