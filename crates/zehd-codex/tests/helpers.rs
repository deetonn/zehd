use zehd_codex::ast::*;
use zehd_codex::error::ParseError;
use zehd_codex::ParseResult;

/// Parse source and assert no errors.
pub fn parse_ok(source: &str) -> ParseResult<'_> {
    let result = zehd_codex::parse(source);
    if !result.is_ok() {
        panic!(
            "expected no errors, got:\n{}",
            format_errors(&result.errors, source)
        );
    }
    result
}

/// Parse source and return result (errors are expected).
pub fn parse_with_errors(source: &str) -> ParseResult<'_> {
    zehd_codex::parse(source)
}

/// Parse a single item from source.
pub fn parse_single_item(source: &str) -> Item {
    let result = parse_ok(source);
    assert_eq!(
        result.program.items.len(),
        1,
        "expected 1 item, got {}",
        result.program.items.len()
    );
    result.program.items.into_iter().next().unwrap()
}

/// Parse source as a single expression statement and return the expression.
pub fn parse_single_expr(source: &str) -> Expr {
    let item = parse_single_item(source);
    match item.kind {
        ItemKind::ExprStmt(es) => es.expr,
        other => panic!("expected ExprStmt, got {:?}", other),
    }
}

/// Format errors for display.
pub fn format_errors(errors: &[ParseError], _source: &str) -> String {
    errors
        .iter()
        .map(|e| format!("  {}", e))
        .collect::<Vec<_>>()
        .join("\n")
}
