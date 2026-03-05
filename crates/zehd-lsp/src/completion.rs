use tower_lsp::lsp_types::*;
use zehd_sigil::scope::{ScopeId, SymbolKind};
use zehd_sigil::types::Type;
use zehd_sigil::ModuleTypes;

use crate::diagnostics::{AnalysisResult, LineIndex};

// ── Context detection ───────────────────────────────────────────

/// What the user is typing at the cursor position.
enum CompletionContext {
    /// After `ident.` or `ident.<partial>` — suggest struct fields.
    DotAccess { ident: String },
    /// Inside `from <cursor>` — suggest module paths.
    ImportPath { partial: String },
    /// Inside `import { <cursor> } from path;` — suggest exports.
    ImportNames { module_path: String },
    /// After `:` in a type annotation position — suggest type names.
    TypeAnnotation,
    /// Default: statement/expression position — keywords + visible symbols.
    General,
}

fn detect_context(source: &str, cursor: usize) -> CompletionContext {
    let before = &source[..cursor];

    // ── Import names: `import { <cursor> } from mod::path;` ─────
    // Find the last `import` before cursor and check if cursor is between { and }.
    if let Some(import_pos) = before.rfind("import") {
        let after_import = &source[import_pos..];
        if let Some(brace_open) = after_import.find('{') {
            let brace_open_abs = import_pos + brace_open;
            // Check if cursor is after the opening brace.
            if cursor > brace_open_abs {
                // Find closing brace — might be after cursor.
                if let Some(brace_close) = after_import[brace_open..].find('}') {
                    let brace_close_abs = brace_open_abs + brace_close;
                    if cursor <= brace_close_abs {
                        // We're inside the braces. Find the module path after `from`.
                        let after_brace = &source[brace_close_abs..];
                        if let Some(from_pos) = after_brace.find("from") {
                            let path_start = brace_close_abs + from_pos + 4;
                            let path_text = source[path_start..]
                                .trim_start()
                                .trim_end_matches(';')
                                .trim();
                            if !path_text.is_empty() {
                                return CompletionContext::ImportNames {
                                    module_path: path_text.to_string(),
                                };
                            }
                        }
                    }
                } else {
                    // No closing brace yet — cursor is inside `import { ... `
                    // Look ahead for `from` to get the module path.
                    // For now, can't resolve path without closing brace + from.
                }
            }
        }
    }

    // ── Import path: `from <cursor>` ─────────────────────────────
    if let Some(from_pos) = before.rfind("from ") {
        let after_from = &before[from_pos + 5..];
        // Only trigger if we're on the same line as `from`.
        if !after_from.contains('\n') {
            return CompletionContext::ImportPath {
                partial: after_from.trim().to_string(),
            };
        }
    }

    // ── Dot access: `ident.` or `ident.<partial>` ────────────────
    // Scan backwards for a dot.
    let before_trimmed = before.trim_end();
    if let Some(dot_pos) = before_trimmed.rfind('.') {
        let after_dot = &before_trimmed[dot_pos + 1..];
        // After the dot should only be identifier chars (partial field name) or empty.
        if after_dot.chars().all(|c| c.is_alphanumeric() || c == '_') {
            // Extract the identifier before the dot.
            let before_dot = &before_trimmed[..dot_pos];
            let ident = extract_ident_backwards(before_dot);
            if !ident.is_empty() {
                return CompletionContext::DotAccess {
                    ident: ident.to_string(),
                };
            }
        }
    }

    // ── Type annotation: after `:` in let/const/param context ────
    if is_type_annotation_position(before) {
        return CompletionContext::TypeAnnotation;
    }

    CompletionContext::General
}

/// Extract an identifier by scanning backwards from the end of `s`.
fn extract_ident_backwards(s: &str) -> &str {
    let s = s.trim_end();
    let end = s.len();
    let start = s
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
        .last()
        .map(|(i, _)| i)
        .unwrap_or(end);
    &s[start..end]
}

/// Heuristic: check if the cursor is in a type annotation position.
/// Looks for patterns like `let x:`, `const y:`, `): `, `fn foo(x:` without a following `=`.
fn is_type_annotation_position(before: &str) -> bool {
    let trimmed = before.trim_end();

    // Find last colon.
    let Some(colon_pos) = trimmed.rfind(':') else {
        return false;
    };

    // Check for `::` (module path) — not a type annotation.
    if colon_pos > 0 && trimmed.as_bytes().get(colon_pos - 1) == Some(&b':') {
        return false;
    }
    if trimmed.as_bytes().get(colon_pos + 1) == Some(&b':') {
        return false;
    }

    let after_colon = &trimmed[colon_pos + 1..];

    // If there's `=`, `{`, or `;` after the colon, we're past the type position.
    if after_colon.contains('=') || after_colon.contains('{') || after_colon.contains(';') {
        return false;
    }

    // Check that before the colon there's something like a variable/param name.
    let before_colon = trimmed[..colon_pos].trim_end();
    if before_colon.is_empty() {
        return false;
    }

    // The last non-space char before `:` should be an identifier char or `)`.
    let last_char = before_colon.chars().last().unwrap();
    last_char.is_alphanumeric() || last_char == '_' || last_char == ')'
}

// ── Completion builders ─────────────────────────────────────────

pub fn completions(
    source: &str,
    position: Position,
    analysis: Option<&AnalysisResult>,
    module_types: &ModuleTypes,
) -> Vec<CompletionItem> {
    let line_index = LineIndex::new(source);
    let Some(cursor) = line_index.position_to_offset(position, source) else {
        return vec![];
    };
    let cursor = cursor as usize;

    let ctx = detect_context(source, cursor);

    match ctx {
        CompletionContext::DotAccess { ident } => {
            if ident == "self" {
                self_field_completions(module_types)
            } else if let Some(analysis) = analysis {
                field_completions_for_ident(&ident, cursor, analysis)
            } else {
                vec![]
            }
        }
        CompletionContext::ImportPath { partial } => {
            module_path_completions(module_types, &partial)
        }
        CompletionContext::ImportNames { module_path } => {
            module_export_completions(module_types, &module_path)
        }
        CompletionContext::TypeAnnotation => {
            let mut items = type_completions();
            if let Some(analysis) = analysis {
                items.extend(scope_type_completions(analysis, cursor));
            }
            items
        }
        CompletionContext::General => {
            let mut items = keyword_completions();
            if let Some(analysis) = analysis {
                items.extend(scope_completions(analysis, cursor));
            }
            items
        }
    }
}

fn keyword_completions() -> Vec<CompletionItem> {
    let keywords = [
        ("let", "Variable declaration (reassignable)"),
        ("const", "Constant declaration"),
        ("fn", "Function declaration"),
        ("if", "Conditional expression"),
        ("else", "Else branch"),
        ("match", "Pattern matching"),
        ("for", "For loop"),
        ("while", "While loop"),
        ("return", "Return from function"),
        ("break", "Break from loop"),
        ("continue", "Continue to next iteration"),
        ("get", "HTTP GET handler"),
        ("post", "HTTP POST handler"),
        ("put", "HTTP PUT handler"),
        ("patch", "HTTP PATCH handler"),
        ("delete", "HTTP DELETE handler"),
        ("import", "Import declaration"),
        ("type", "Type alias"),
        ("enum", "Enum declaration"),
        ("init", "Startup configuration block"),
        ("error", "Error handler block"),
        ("true", "Boolean true"),
        ("false", "Boolean false"),
        ("None", "Option::None"),
        ("Some", "Option::Some"),
        ("Ok", "Result::Ok"),
        ("Err", "Result::Err"),
        ("self", "Request context"),
    ];

    keywords
        .iter()
        .map(|(kw, detail)| CompletionItem {
            label: kw.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            ..Default::default()
        })
        .collect()
}

fn scope_completions(analysis: &AnalysisResult, cursor: usize) -> Vec<CompletionItem> {
    let cursor_u32 = cursor as u32;
    // Collect the best symbol for each name. The resolver creates placeholder
    // symbols with Type::Var(0) in early scopes; the checker later defines the
    // real symbol (with concrete types) in higher-numbered scopes. We keep the
    // version with the most informative type.
    let mut best: std::collections::HashMap<String, &zehd_sigil::scope::Symbol> =
        std::collections::HashMap::new();

    for scope_id in 0..analysis.scopes.scope_count() {
        for (name, symbol) in analysis.scopes.symbols(scope_id as ScopeId) {
            if symbol.defined_at.start <= cursor_u32 {
                let dominated = best.get(name.as_str()).is_some_and(|prev| {
                    has_concrete_type(prev)
                });
                // Replace if we haven't seen this name, or the previous entry
                // was a placeholder and this one is concrete.
                if !best.contains_key(name.as_str())
                    || (!dominated && has_concrete_type(symbol))
                {
                    best.insert(name.clone(), symbol);
                }
            }
        }
    }

    best.into_iter()
        .map(|(name, symbol)| CompletionItem {
            label: name,
            kind: Some(completion_item_kind(symbol)),
            detail: Some(display_type(&symbol.ty)),
            ..Default::default()
        })
        .collect()
}

/// Returns true if the symbol has a resolved (non-placeholder) type.
pub(crate) fn has_concrete_type(symbol: &zehd_sigil::scope::Symbol) -> bool {
    !matches!(symbol.ty, Type::Var(_) | Type::Error)
}

/// Pick a CompletionItemKind based on both the symbol kind and its resolved type.
fn completion_item_kind(symbol: &zehd_sigil::scope::Symbol) -> CompletionItemKind {
    match symbol.kind {
        SymbolKind::Function => CompletionItemKind::FUNCTION,
        SymbolKind::Variable | SymbolKind::Parameter => {
            // A variable/const holding a function should show the function icon.
            if matches!(symbol.ty, Type::Function(_)) {
                CompletionItemKind::FUNCTION
            } else {
                CompletionItemKind::VARIABLE
            }
        }
        SymbolKind::TypeDef => CompletionItemKind::STRUCT,
        SymbolKind::EnumDef => CompletionItemKind::ENUM,
        SymbolKind::EnumVariant => CompletionItemKind::ENUM_MEMBER,
        SymbolKind::Import => CompletionItemKind::MODULE,
    }
}

/// Display a type, recursively replacing unresolved type variables with "unknown".
pub(crate) fn display_type(ty: &Type) -> String {
    match ty {
        Type::Var(_) | Type::Error => "unknown".to_string(),
        Type::Int => "int".to_string(),
        Type::Float => "float".to_string(),
        Type::String => "string".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Time => "time".to_string(),
        Type::Unit => "()".to_string(),
        Type::Never => "never".to_string(),
        Type::Option(inner) => format!("Option<{}>", display_type(inner)),
        Type::Result(ok, err) => {
            format!("Result<{}, {}>", display_type(ok), display_type(err))
        }
        Type::List(elem) => format!("List<{}>", display_type(elem)),
        Type::Map(k, v) => format!("Map<{}, {}>", display_type(k), display_type(v)),
        Type::Struct(s) => {
            if let Some(name) = &s.name {
                name.clone()
            } else {
                let fields: Vec<String> = s
                    .fields
                    .iter()
                    .map(|(name, ty)| format!("{name}: {}", display_type(ty)))
                    .collect();
                format!("{{ {} }}", fields.join(", "))
            }
        }
        Type::Enum(e) => e.name.clone(),
        Type::Function(ft) => {
            let params: Vec<String> = ft.params.iter().map(|p| display_type(p)).collect();
            format!("({}) => {}", params.join(", "), display_type(&ft.return_type))
        }
    }
}

fn scope_type_completions(analysis: &AnalysisResult, cursor: usize) -> Vec<CompletionItem> {
    let cursor_u32 = cursor as u32;
    let mut best: std::collections::HashMap<String, &zehd_sigil::scope::Symbol> =
        std::collections::HashMap::new();

    for scope_id in 0..analysis.scopes.scope_count() {
        for (name, symbol) in analysis.scopes.symbols(scope_id as ScopeId) {
            if symbol.defined_at.start <= cursor_u32
                && matches!(symbol.kind, SymbolKind::TypeDef | SymbolKind::EnumDef)
            {
                if !best.contains_key(name.as_str())
                    || (!has_concrete_type(best[name.as_str()]) && has_concrete_type(symbol))
                {
                    best.insert(name.clone(), symbol);
                }
            }
        }
    }

    best.into_iter()
        .map(|(name, symbol)| {
            let kind = match symbol.kind {
                SymbolKind::EnumDef => CompletionItemKind::ENUM,
                _ => CompletionItemKind::STRUCT,
            };
            CompletionItem {
                label: name,
                kind: Some(kind),
                detail: Some(display_type(&symbol.ty)),
                ..Default::default()
            }
        })
        .collect()
}

fn field_completions_for_ident(
    ident: &str,
    cursor: usize,
    analysis: &AnalysisResult,
) -> Vec<CompletionItem> {
    let cursor_u32 = cursor as u32;

    // Find the best symbol across all scopes (prefer concrete types).
    let mut best: Option<&zehd_sigil::scope::Symbol> = None;
    for scope_id in 0..analysis.scopes.scope_count() {
        for (name, symbol) in analysis.scopes.symbols(scope_id as ScopeId) {
            if name == ident && symbol.defined_at.start <= cursor_u32 {
                if best.is_none() || (!has_concrete_type(best.unwrap()) && has_concrete_type(symbol))
                {
                    best = Some(symbol);
                }
            }
        }
    }

    best.map(|sym| field_completions(&sym.ty)).unwrap_or_default()
}

/// Build completions for `self.` — the implicit route context.
/// Mirrors the RouteContext struct synthesized in the checker.
fn self_field_completions(module_types: &ModuleTypes) -> Vec<CompletionItem> {
    let http = module_types.get("std::http");
    let request_ty = http
        .and_then(|m| m.get("Request"))
        .cloned()
        .unwrap_or(Type::Error);
    let response_ty = http
        .and_then(|m| m.get("Response"))
        .cloned()
        .unwrap_or(Type::Error);

    let self_ty = Type::Struct(zehd_sigil::types::StructType {
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
    });

    field_completions(&self_ty)
}

fn field_completions(ty: &Type) -> Vec<CompletionItem> {
    match ty {
        Type::Struct(st) => st
            .fields
            .iter()
            .map(|(name, field_ty)| CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(display_type(field_ty)),
                ..Default::default()
            })
            .collect(),
        _ => vec![],
    }
}

fn module_path_completions(
    module_types: &ModuleTypes,
    partial: &str,
) -> Vec<CompletionItem> {
    module_types
        .keys()
        .filter(|path| {
            if partial.is_empty() {
                true
            } else {
                path.starts_with(partial)
            }
        })
        .map(|path| CompletionItem {
            label: path.clone(),
            kind: Some(CompletionItemKind::MODULE),
            ..Default::default()
        })
        .collect()
}

fn module_export_completions(
    module_types: &ModuleTypes,
    module_path: &str,
) -> Vec<CompletionItem> {
    let Some(exports) = module_types.get(module_path) else {
        return vec![];
    };
    exports
        .iter()
        .map(|(name, ty)| CompletionItem {
            label: name.clone(),
            kind: Some(match ty {
                Type::Function(_) => CompletionItemKind::FUNCTION,
                Type::Struct(_) => CompletionItemKind::STRUCT,
                Type::Enum(_) => CompletionItemKind::ENUM,
                _ => CompletionItemKind::VARIABLE,
            }),
            detail: Some(display_type(ty)),
            ..Default::default()
        })
        .collect()
}

fn type_completions() -> Vec<CompletionItem> {
    let types = [
        ("int", "Integer type"),
        ("float", "Floating-point type"),
        ("string", "String type"),
        ("bool", "Boolean type"),
        ("time", "Time type (milliseconds)"),
        ("Option", "Optional value (Option<T>)"),
        ("Result", "Result type (Result<T, E>)"),
        ("List", "List type (List<T>)"),
        ("Map", "Map type (Map<K, V>)"),
    ];

    types
        .iter()
        .map(|(name, detail)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some(detail.to_string()),
            ..Default::default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_general_context() {
        let source = "let x = 42;\n";
        let ctx = detect_context(source, source.len());
        assert!(matches!(ctx, CompletionContext::General));
    }

    #[test]
    fn detect_dot_access() {
        let source = "user.";
        let ctx = detect_context(source, source.len());
        assert!(matches!(ctx, CompletionContext::DotAccess { ident } if ident == "user"));
    }

    #[test]
    fn detect_dot_access_with_partial() {
        let source = "user.na";
        let ctx = detect_context(source, source.len());
        assert!(matches!(ctx, CompletionContext::DotAccess { ident } if ident == "user"));
    }

    #[test]
    fn detect_import_path() {
        let source = "import { info } from std";
        let ctx = detect_context(source, source.len());
        assert!(matches!(ctx, CompletionContext::ImportPath { partial } if partial == "std"));
    }

    #[test]
    fn detect_import_path_empty() {
        let source = "import { info } from ";
        let ctx = detect_context(source, source.len());
        assert!(matches!(ctx, CompletionContext::ImportPath { partial } if partial.is_empty()));
    }

    #[test]
    fn detect_type_annotation() {
        let source = "let x: ";
        let ctx = detect_context(source, source.len());
        assert!(matches!(ctx, CompletionContext::TypeAnnotation));
    }

    #[test]
    fn detect_type_annotation_fn_param() {
        let source = "fn foo(x: ";
        let ctx = detect_context(source, source.len());
        assert!(matches!(ctx, CompletionContext::TypeAnnotation));
    }

    #[test]
    fn detect_not_type_annotation_after_equals() {
        let source = "let x: int = ";
        let ctx = detect_context(source, source.len());
        assert!(matches!(ctx, CompletionContext::General));
    }

    #[test]
    fn detect_not_type_annotation_module_path() {
        let source = "import { info } from std::";
        let ctx = detect_context(source, source.len());
        // Should be ImportPath, not TypeAnnotation.
        assert!(matches!(ctx, CompletionContext::ImportPath { .. }));
    }

    #[test]
    fn keyword_completions_not_empty() {
        let items = keyword_completions();
        assert!(!items.is_empty());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"let"));
        assert!(labels.contains(&"const"));
        assert!(labels.contains(&"fn"));
        assert!(labels.contains(&"get"));
    }

    #[test]
    fn type_completions_not_empty() {
        let items = type_completions();
        assert!(!items.is_empty());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"int"));
        assert!(labels.contains(&"string"));
        assert!(labels.contains(&"Option"));
    }

    #[test]
    fn module_path_completions_filters() {
        let mut mt = ModuleTypes::new();
        mt.insert("std".into(), Default::default());
        mt.insert("std::log".into(), Default::default());
        mt.insert("lib::math".into(), Default::default());

        let items = module_path_completions(&mt, "std");
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"std"));
        assert!(labels.contains(&"std::log"));
        assert!(!labels.contains(&"lib::math"));
    }

    #[test]
    fn module_path_completions_all() {
        let mut mt = ModuleTypes::new();
        mt.insert("std".into(), Default::default());
        mt.insert("std::log".into(), Default::default());

        let items = module_path_completions(&mt, "");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn module_export_completions_returns_exports() {
        let mut mt = ModuleTypes::new();
        let mut exports = std::collections::HashMap::new();
        exports.insert(
            "info".to_string(),
            Type::Function(zehd_sigil::types::FunctionType {
                type_params: vec![],
                type_param_vars: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::Unit),
            }),
        );
        mt.insert("std::log".into(), exports);

        let items = module_export_completions(&mt, "std::log");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "info");
        assert_eq!(items[0].kind, Some(CompletionItemKind::FUNCTION));
    }

    #[test]
    fn module_export_completions_unknown_module() {
        let mt = ModuleTypes::new();
        let items = module_export_completions(&mt, "unknown");
        assert!(items.is_empty());
    }

    #[test]
    fn field_completions_for_struct_type() {
        let ty = Type::Struct(zehd_sigil::types::StructType {
            name: Some("User".into()),
            fields: vec![
                ("name".into(), Type::String),
                ("age".into(), Type::Int),
            ],
            type_params: vec![],
        });
        let items = field_completions(&ty);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].label, "name");
        assert_eq!(items[0].kind, Some(CompletionItemKind::FIELD));
        assert_eq!(items[1].label, "age");
    }

    #[test]
    fn field_completions_non_struct() {
        let items = field_completions(&Type::Int);
        assert!(items.is_empty());
    }

    #[test]
    fn extract_ident_backwards_simple() {
        assert_eq!(extract_ident_backwards("foo"), "foo");
        assert_eq!(extract_ident_backwards("bar.baz"), "baz");
        assert_eq!(extract_ident_backwards("let x = user"), "user");
        assert_eq!(extract_ident_backwards("  hello  "), "hello");
    }

    #[test]
    fn completions_general_without_analysis() {
        let source = "le";
        let items = completions(source, Position::new(0, 2), None, &ModuleTypes::new());
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"let"));
    }

    #[test]
    fn completions_import_path() {
        let source = "import { info } from std";
        let mut mt = ModuleTypes::new();
        mt.insert("std".into(), Default::default());
        mt.insert("std::log".into(), Default::default());

        let items = completions(source, Position::new(0, 24), None, &mt);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"std"));
        assert!(labels.contains(&"std::log"));
    }

    #[test]
    fn detect_import_names() {
        let source = "import {  } from std::log;";
        // Cursor at position 9 (inside the braces).
        let ctx = detect_context(source, 9);
        assert!(
            matches!(ctx, CompletionContext::ImportNames { module_path } if module_path == "std::log")
        );
    }
}
