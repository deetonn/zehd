use zehd_codex::ast::NodeId;
use zehd_sigil::checker::TypeTable;
use zehd_sigil::error::TypeError;
use zehd_sigil::types::Type;
use zehd_sigil::CheckResult;

/// Parse and type-check source code in one step.
pub fn check_ok(source: &str) -> CheckResult {
    let parse_result = zehd_codex::parse(source);
    if !parse_result.is_ok() {
        panic!(
            "parse errors:\n{}",
            format_errors_parse(&parse_result.errors, source)
        );
    }
    let result = zehd_sigil::check(&parse_result.program, source);
    if result.has_errors() {
        panic!(
            "type errors:\n{}",
            format_errors(&result.errors, source)
        );
    }
    result
}

/// Parse and type-check, expecting type errors.
pub fn check_with_errors(source: &str) -> CheckResult {
    let parse_result = zehd_codex::parse(source);
    if !parse_result.is_ok() {
        panic!(
            "parse errors:\n{}",
            format_errors_parse(&parse_result.errors, source)
        );
    }
    zehd_sigil::check(&parse_result.program, source)
}

/// Parse and type-check, return just the type table.
pub fn check_types(source: &str) -> TypeTable {
    check_ok(source).types
}

/// Find the type of a specific node by its NodeId.
pub fn type_of(types: &TypeTable, id: NodeId) -> &Type {
    types.get(&id).expect("no type for node")
}

/// Check that the result contains an error with the given code.
pub fn has_error_code(result: &CheckResult, code: &str) -> bool {
    result.errors.iter().any(|e| e.code.to_string() == code)
}

/// Format type errors for display.
pub fn format_errors(errors: &[TypeError], _source: &str) -> String {
    errors
        .iter()
        .map(|e| format!("  {}", e))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_errors_parse(errors: &[zehd_codex::error::ParseError], _source: &str) -> String {
    errors
        .iter()
        .map(|e| format!("  {}", e))
        .collect::<Vec<_>>()
        .join("\n")
}
