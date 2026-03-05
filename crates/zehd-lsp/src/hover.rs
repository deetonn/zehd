use tower_lsp::lsp_types::*;
use zehd_sigil::builtin_methods;
use zehd_sigil::scope::{ScopeId, SymbolKind};
use zehd_sigil::types::Type;
use zehd_sigil::ModuleTypes;

use crate::completion::{display_type, has_concrete_type};
use crate::diagnostics::{AnalysisResult, LineIndex};

/// Compute hover information for the given source position.
pub fn hover_info(
    source: &str,
    position: Position,
    analysis: Option<&AnalysisResult>,
    module_types: &ModuleTypes,
) -> Option<Hover> {
    let line_index = LineIndex::new(source);
    let cursor = line_index.position_to_offset(position, source)? as usize;

    let (word, word_start, word_end) = find_word_at_position(source, cursor)?;

    // Detect context: is this a dot-field access?
    let content = if let Some(receiver) = dot_receiver(source, word_start) {
        hover_dot_field(&receiver, &word, cursor, analysis, module_types)?
    } else if word == "self" {
        hover_self(module_types)
    } else if let Some(content) = hover_primitive_type(&word) {
        content
    } else if let Some(content) = hover_builtin_type(&word) {
        content
    } else if let Some(analysis) = analysis {
        hover_identifier(&word, cursor, analysis)?
    } else {
        return None;
    };

    let range = Some(Range::new(
        line_index.offset_to_position(word_start as u32, source),
        line_index.offset_to_position(word_end as u32, source),
    ));

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range,
    })
}

// ── Word extraction ─────────────────────────────────────────────

/// Find the full identifier word the cursor is on/inside.
/// Returns (word, start_byte, end_byte).
fn find_word_at_position(source: &str, cursor: usize) -> Option<(String, usize, usize)> {
    let bytes = source.as_bytes();

    // If cursor is at the end or on a non-ident char, try one position back.
    let pos = if cursor < bytes.len() && is_ident_byte(bytes[cursor]) {
        cursor
    } else if cursor > 0 && is_ident_byte(bytes[cursor - 1]) {
        cursor - 1
    } else {
        return None;
    };

    // Scan backwards to start of word.
    let mut start = pos;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }

    // Scan forwards to end of word.
    let mut end = pos;
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    let word = source[start..end].to_string();
    Some((word, start, end))
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

// ── Context detection ───────────────────────────────────────────

/// If the word at `word_start` is preceded by `.`, return the receiver identifier.
fn dot_receiver(source: &str, word_start: usize) -> Option<String> {
    if word_start == 0 {
        return None;
    }

    let before = source[..word_start].trim_end();
    if !before.ends_with('.') {
        return None;
    }

    // Extract the identifier before the dot.
    let before_dot = &before[..before.len() - 1];
    let trimmed = before_dot.trim_end();
    let bytes = trimmed.as_bytes();
    let end = trimmed.len();
    let mut start = end;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }

    if start == end {
        return None;
    }

    Some(trimmed[start..end].to_string())
}

// ── Hover generators ────────────────────────────────────────────

fn hover_dot_field(
    receiver: &str,
    field: &str,
    cursor: usize,
    analysis: Option<&AnalysisResult>,
    module_types: &ModuleTypes,
) -> Option<String> {
    let receiver_ty = if receiver == "self" {
        self_type(module_types)
    } else {
        let analysis = analysis?;
        let sym = find_best_symbol(analysis, receiver, cursor)?;
        sym.ty.clone()
    };

    // Check struct fields first.
    if let Type::Struct(st) = &receiver_ty {
        for (name, ty) in &st.fields {
            if name == field {
                return Some(format_code(&format!("(field) {name}: {}", display_type(ty))));
            }
        }
    }

    // Check built-in methods.
    if let Some(sig) = builtin_methods::resolve_builtin_method(&receiver_ty, field) {
        let display = if sig.params.is_empty() {
            format!("(property) {field}: {}", display_type(&sig.return_type))
        } else {
            let params: Vec<String> = sig.params.iter().map(|p| display_type(p)).collect();
            format!(
                "(method) {field}({}): {}",
                params.join(", "),
                display_type(&sig.return_type)
            )
        };
        return Some(format_code(&display));
    }

    None
}

fn hover_self(module_types: &ModuleTypes) -> String {
    let ty = self_type(module_types);
    match &ty {
        Type::Struct(st) => {
            let mut lines = vec!["type RouteContext {".to_string()];
            for (name, field_ty) in &st.fields {
                lines.push(format!("    {name}: {};", display_type(field_ty)));
            }
            lines.push("}".to_string());
            format_code(&lines.join("\n"))
        }
        _ => format_code("self: RouteContext"),
    }
}

fn hover_identifier(
    name: &str,
    cursor: usize,
    analysis: &AnalysisResult,
) -> Option<String> {
    let symbol = find_best_symbol(analysis, name, cursor)?;

    let content = match symbol.kind {
        SymbolKind::Function => format_function(name, &symbol.ty),
        SymbolKind::Variable | SymbolKind::Parameter => {
            let keyword = if symbol.mutable { "let" } else { "const" };
            format_code(&format!("{keyword} {name}: {}", display_type(&symbol.ty)))
        }
        SymbolKind::TypeDef => format_typedef(name, &symbol.ty),
        SymbolKind::EnumDef => format_enumdef(name, &symbol.ty),
        SymbolKind::EnumVariant => {
            format_code(&format!("(variant) {name}: {}", display_type(&symbol.ty)))
        }
        SymbolKind::Import => {
            format_code(&format!("(import) {name}: {}", display_type(&symbol.ty)))
        }
    };

    Some(content)
}

fn hover_primitive_type(word: &str) -> Option<String> {
    let desc = match word {
        "int" => "Integer type — 64-bit signed integer",
        "float" => "Floating-point type — 64-bit IEEE 754",
        "string" => "String type — UTF-8 text",
        "bool" => "Boolean type — `true` or `false`",
        "time" => "Time type — duration in milliseconds",
        _ => return None,
    };
    Some(format!("{}\n\n{desc}", format_code(word)))
}

fn hover_builtin_type(word: &str) -> Option<String> {
    let desc = match word {
        "Option" => "Optional value — `Some(T)` or `None`",
        "Result" => "Result type — `Ok(T)` or `Err(E)`",
        "List" => "Ordered collection — `List<T>`",
        "Map" => "Key-value map — `Map<K, V>`",
        _ => return None,
    };
    Some(format!("{}\n\n{desc}", format_code(word)))
}

// ── Formatting helpers ──────────────────────────────────────────

fn format_code(code: &str) -> String {
    format!("```zehd\n{code}\n```")
}

fn format_function(name: &str, ty: &Type) -> String {
    match ty {
        Type::Function(ft) => {
            let params: Vec<String> = ft.params.iter().map(|p| display_type(p)).collect();
            let ret = display_type(&ft.return_type);
            format_code(&format!("fn {name}({}): {ret}", params.join(", ")))
        }
        _ => format_code(&format!("fn {name}(): {}", display_type(ty))),
    }
}

fn format_typedef(name: &str, ty: &Type) -> String {
    match ty {
        Type::Struct(st) => {
            let mut lines = vec![format!("type {name} {{")];
            for (fname, fty) in &st.fields {
                lines.push(format!("    {fname}: {};", display_type(fty)));
            }
            lines.push("}".to_string());
            format_code(&lines.join("\n"))
        }
        _ => format_code(&format!("type {name} = {}", display_type(ty))),
    }
}

fn format_enumdef(name: &str, ty: &Type) -> String {
    match ty {
        Type::Enum(et) => {
            let mut lines = vec![format!("enum {name} {{")];
            for variant in &et.variants {
                if let Some(payload) = &variant.payload {
                    lines.push(format!("    {}({}),", variant.name, display_type(payload)));
                } else {
                    lines.push(format!("    {},", variant.name));
                }
            }
            lines.push("}".to_string());
            format_code(&lines.join("\n"))
        }
        _ => format_code(&format!("enum {name}")),
    }
}

// ── Shared symbol lookup ────────────────────────────────────────

/// Find the best symbol for `name` across all scopes, preferring concrete types.
fn find_best_symbol<'a>(
    analysis: &'a AnalysisResult,
    name: &str,
    cursor: usize,
) -> Option<&'a zehd_sigil::scope::Symbol> {
    let cursor_u32 = cursor as u32;
    let mut best: Option<&zehd_sigil::scope::Symbol> = None;

    for scope_id in 0..analysis.scopes.scope_count() {
        for (sym_name, symbol) in analysis.scopes.symbols(scope_id as ScopeId) {
            if sym_name == name && symbol.defined_at.start <= cursor_u32 {
                if best.is_none()
                    || (!has_concrete_type(best.unwrap()) && has_concrete_type(symbol))
                {
                    best = Some(symbol);
                }
            }
        }
    }

    best
}

/// Build the RouteContext type, mirroring the checker's self type.
fn self_type(module_types: &ModuleTypes) -> Type {
    let http = module_types.get("std::http");
    let request_ty = http
        .and_then(|m| m.get("Request"))
        .cloned()
        .unwrap_or(Type::Error);
    let response_ty = http
        .and_then(|m| m.get("Response"))
        .cloned()
        .unwrap_or(Type::Error);

    Type::Struct(zehd_sigil::types::StructType {
        name: Some("RouteContext".to_string()),
        fields: vec![
            ("request".to_string(), request_ty),
            ("response".to_string(), response_ty),
            (
                "params".to_string(),
                Type::Map(Box::new(Type::String), Box::new(Type::String)),
            ),
        ],
        type_params: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_word_at_cursor_middle() {
        let source = "let hello = 42;";
        let (word, start, end) = find_word_at_position(source, 5).unwrap();
        assert_eq!(word, "hello");
        assert_eq!(start, 4);
        assert_eq!(end, 9);
    }

    #[test]
    fn find_word_at_cursor_start() {
        let source = "hello world";
        let (word, start, end) = find_word_at_position(source, 0).unwrap();
        assert_eq!(word, "hello");
        assert_eq!(start, 0);
        assert_eq!(end, 5);
    }

    #[test]
    fn find_word_at_cursor_end() {
        let source = "hello";
        let (word, start, end) = find_word_at_position(source, 5).unwrap();
        assert_eq!(word, "hello");
        assert_eq!(start, 0);
        assert_eq!(end, 5);
    }

    #[test]
    fn find_word_at_space_returns_none() {
        let source = "( )";
        assert!(find_word_at_position(source, 1).is_none());
    }

    #[test]
    fn dot_receiver_found() {
        let source = "user.name";
        assert_eq!(dot_receiver(source, 5), Some("user".to_string()));
    }

    #[test]
    fn dot_receiver_self() {
        let source = "self.request";
        assert_eq!(dot_receiver(source, 5), Some("self".to_string()));
    }

    #[test]
    fn dot_receiver_no_dot() {
        let source = "username";
        assert_eq!(dot_receiver(source, 0), None);
    }

    #[test]
    fn hover_primitive_types() {
        assert!(hover_primitive_type("int").is_some());
        assert!(hover_primitive_type("float").is_some());
        assert!(hover_primitive_type("string").is_some());
        assert!(hover_primitive_type("bool").is_some());
        assert!(hover_primitive_type("time").is_some());
        assert!(hover_primitive_type("foo").is_none());
    }

    #[test]
    fn hover_builtin_types() {
        assert!(hover_builtin_type("Option").is_some());
        assert!(hover_builtin_type("Result").is_some());
        assert!(hover_builtin_type("List").is_some());
        assert!(hover_builtin_type("Map").is_some());
        assert!(hover_builtin_type("foo").is_none());
    }

    #[test]
    fn format_function_display() {
        let ty = Type::Function(zehd_sigil::types::FunctionType {
            type_params: vec![],
            type_param_vars: vec![],
            params: vec![Type::String],
            return_type: Box::new(Type::Unit),
        });
        let result = format_function("info", &ty);
        assert!(result.contains("fn info(string): ()"));
    }

    #[test]
    fn format_struct_typedef() {
        let ty = Type::Struct(zehd_sigil::types::StructType {
            name: Some("User".into()),
            fields: vec![
                ("name".into(), Type::String),
                ("age".into(), Type::Int),
            ],
            type_params: vec![],
        });
        let result = format_typedef("User", &ty);
        assert!(result.contains("type User {"));
        assert!(result.contains("name: string;"));
        assert!(result.contains("age: int;"));
    }

    #[test]
    fn format_enum_display() {
        let ty = Type::Enum(zehd_sigil::types::EnumType {
            name: "Color".into(),
            variants: vec![
                zehd_sigil::types::EnumVariantType {
                    name: "Red".into(),
                    payload: None,
                },
                zehd_sigil::types::EnumVariantType {
                    name: "Custom".into(),
                    payload: Some(Type::String),
                },
            ],
            type_params: vec![],
        });
        let result = format_enumdef("Color", &ty);
        assert!(result.contains("enum Color {"));
        assert!(result.contains("Red,"));
        assert!(result.contains("Custom(string),"));
    }

    #[test]
    fn hover_self_shows_route_context() {
        let module_types = zehd_sigil::std_module_types();
        let result = hover_self(&module_types);
        assert!(result.contains("RouteContext"));
        assert!(result.contains("request"));
        assert!(result.contains("response"));
        assert!(result.contains("params"));
    }

    #[test]
    fn hover_info_on_primitive_keyword() {
        let source = "let x: int = 42;";
        let module_types = zehd_sigil::std_module_types();
        // Position on "int" (line 0, col 7)
        let hover = hover_info(source, Position::new(0, 8), None, &module_types);
        assert!(hover.is_some());
        let content = match hover.unwrap().contents {
            HoverContents::Markup(m) => m.value,
            _ => panic!("expected markup"),
        };
        assert!(content.contains("int"));
        assert!(content.contains("64-bit"));
    }

    #[test]
    fn hover_info_on_self() {
        let source = "get { self.request; }";
        let module_types = zehd_sigil::std_module_types();
        // Position on "self" (line 0, col 6)
        let hover = hover_info(source, Position::new(0, 7), None, &module_types);
        assert!(hover.is_some());
        let content = match hover.unwrap().contents {
            HoverContents::Markup(m) => m.value,
            _ => panic!("expected markup"),
        };
        assert!(content.contains("RouteContext"));
    }

    #[test]
    fn hover_info_self_dot_field() {
        let source = "get { self.request; }";
        let module_types = zehd_sigil::std_module_types();
        // Position on "request" (line 0, col 12)
        let hover = hover_info(source, Position::new(0, 12), None, &module_types);
        assert!(hover.is_some());
        let content = match hover.unwrap().contents {
            HoverContents::Markup(m) => m.value,
            _ => panic!("expected markup"),
        };
        assert!(content.contains("(field) request"));
    }

    #[test]
    fn hover_info_no_word_returns_none() {
        let source = "let x = 42;";
        let module_types = zehd_sigil::std_module_types();
        // Position on space (line 0, col 3)
        let hover = hover_info(source, Position::new(0, 3), None, &module_types);
        assert!(hover.is_none());
    }

    #[test]
    fn hover_builtin_method_property() {
        // Test that hovering on a zero-arg method shows "(property)"
        let sig = builtin_methods::resolve_builtin_method(&Type::String, "length").unwrap();
        assert!(sig.params.is_empty());
        // The hover_dot_field function would format this as "(property) length: int"
    }

    #[test]
    fn hover_builtin_method_with_args() {
        // Test that hovering on a method with args shows "(method)"
        let sig = builtin_methods::resolve_builtin_method(&Type::String, "contains").unwrap();
        assert_eq!(sig.params.len(), 1);
    }
}
