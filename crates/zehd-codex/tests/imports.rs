mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn destructured_import_single() {
    let item = parse_single_item("import { proxy } from std;");
    match &item.kind {
        ItemKind::Import(imp) => {
            assert_eq!(imp.names.len(), 1);
            assert_eq!(imp.names[0].name.name, "proxy");
            assert_eq!(imp.path.segments.len(), 1);
            assert_eq!(imp.path.segments[0].name, "std");
        }
        other => panic!("expected Import, got {:?}", other),
    }
}

#[test]
fn destructured_import_multiple() {
    let item = parse_single_item("import { use, rateLimit } from std;");
    match &item.kind {
        ItemKind::Import(imp) => {
            assert_eq!(imp.names.len(), 2);
            assert_eq!(imp.names[0].name.name, "use");
            assert_eq!(imp.names[1].name.name, "rateLimit");
            assert_eq!(imp.path.segments[0].name, "std");
        }
        other => panic!("expected Import, got {:?}", other),
    }
}

#[test]
fn destructured_import_with_path() {
    let item = parse_single_item("import { validate } from std::validation;");
    match &item.kind {
        ItemKind::Import(imp) => {
            assert_eq!(imp.names.len(), 1);
            assert_eq!(imp.names[0].name.name, "validate");
            assert_eq!(imp.path.segments.len(), 2);
            assert_eq!(imp.path.segments[0].name, "std");
            assert_eq!(imp.path.segments[1].name, "validation");
        }
        other => panic!("expected Import, got {:?}", other),
    }
}

#[test]
fn destructured_import_trailing_comma() {
    let item = parse_single_item("import { use, rateLimit, } from std;");
    match &item.kind {
        ItemKind::Import(imp) => {
            assert_eq!(imp.names.len(), 2);
        }
        other => panic!("expected Import, got {:?}", other),
    }
}

#[test]
fn non_destructured_import() {
    let item = parse_single_item("import std::types::Response;");
    match &item.kind {
        ItemKind::Import(imp) => {
            assert_eq!(imp.names.len(), 1);
            assert_eq!(imp.names[0].name.name, "Response");
            assert_eq!(imp.path.segments.len(), 3);
            assert_eq!(imp.path.segments[0].name, "std");
            assert_eq!(imp.path.segments[1].name, "types");
            assert_eq!(imp.path.segments[2].name, "Response");
        }
        other => panic!("expected Import, got {:?}", other),
    }
}

#[test]
fn multiple_imports() {
    let result = parse_ok(
        "import { proxy } from std;
         import { validate } from std::validation;",
    );
    assert_eq!(result.program.items.len(), 2);
    assert!(matches!(&result.program.items[0].kind, ItemKind::Import(_)));
    assert!(matches!(&result.program.items[1].kind, ItemKind::Import(_)));
}
