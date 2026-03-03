use zehd_rune::chunk::Chunk;
use zehd_rune::error::CompileError;
use zehd_rune::module::CompiledModule;
use zehd_rune::op::{decode_ops, Instruction};
use zehd_rune::CompileResult;

/// Parse, type-check, and compile source code in one step.
/// Panics on parse or type errors.
pub fn compile_ok(source: &str) -> CompileResult {
    let parse_result = zehd_codex::parse(source);
    if !parse_result.is_ok() {
        panic!(
            "parse errors:\n{}",
            format_parse_errors(&parse_result.errors)
        );
    }
    let check_result = zehd_sigil::check(&parse_result.program, source);
    if check_result.has_errors() {
        panic!(
            "type errors:\n{}",
            format_type_errors(&check_result.errors)
        );
    }
    let result = zehd_rune::compile(&parse_result.program, check_result);
    if result.has_errors() {
        panic!(
            "compile errors:\n{}",
            format_compile_errors(&result.errors)
        );
    }
    result
}

/// Parse, type-check, and compile, expecting compile errors.
#[allow(dead_code)]
pub fn compile_with_errors(source: &str) -> CompileResult {
    let parse_result = zehd_codex::parse(source);
    if !parse_result.is_ok() {
        panic!(
            "parse errors:\n{}",
            format_parse_errors(&parse_result.errors)
        );
    }
    let check_result = zehd_sigil::check(&parse_result.program, source);
    if check_result.has_errors() {
        panic!(
            "type errors:\n{}",
            format_type_errors(&check_result.errors)
        );
    }
    zehd_rune::compile(&parse_result.program, check_result)
}

/// Get the compiled module from a source string.
pub fn compile_module(source: &str) -> CompiledModule {
    compile_ok(source).module
}

/// Decode bytecode from a chunk into instructions.
pub fn decode(chunk: &Chunk) -> Vec<Instruction> {
    decode_ops(&chunk.code)
}

/// Check that a result has a compile error with the given code.
#[allow(dead_code)]
pub fn has_error_code(result: &CompileResult, code: &str) -> bool {
    result.errors.iter().any(|e| e.code.to_string() == code)
}

fn format_parse_errors(errors: &[zehd_codex::error::ParseError]) -> String {
    errors
        .iter()
        .map(|e| format!("  {e}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_type_errors(errors: &[zehd_sigil::error::TypeError]) -> String {
    errors
        .iter()
        .map(|e| format!("  {e}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
fn format_compile_errors(errors: &[CompileError]) -> String {
    errors
        .iter()
        .map(|e| format!("  {e}"))
        .collect::<Vec<_>>()
        .join("\n")
}
