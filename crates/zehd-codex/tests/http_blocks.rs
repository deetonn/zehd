mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn get_block() {
    let item = parse_single_item("get { return 42; }");
    match &item.kind {
        ItemKind::HttpBlock(hb) => {
            assert_eq!(hb.method, HttpMethod::Get);
            assert_eq!(hb.body.stmts.len(), 1);
        }
        other => panic!("expected HttpBlock, got {:?}", other),
    }
}

#[test]
fn post_block() {
    let item = parse_single_item("post { return 42; }");
    match &item.kind {
        ItemKind::HttpBlock(hb) => {
            assert_eq!(hb.method, HttpMethod::Post);
        }
        other => panic!("expected HttpBlock, got {:?}", other),
    }
}

#[test]
fn put_block() {
    let item = parse_single_item("put { return 42; }");
    match &item.kind {
        ItemKind::HttpBlock(hb) => {
            assert_eq!(hb.method, HttpMethod::Put);
        }
        other => panic!("expected HttpBlock, got {:?}", other),
    }
}

#[test]
fn patch_block() {
    let item = parse_single_item("patch { return 42; }");
    match &item.kind {
        ItemKind::HttpBlock(hb) => {
            assert_eq!(hb.method, HttpMethod::Patch);
        }
        other => panic!("expected HttpBlock, got {:?}", other),
    }
}

#[test]
fn delete_block() {
    let item = parse_single_item("delete { return 42; }");
    match &item.kind {
        ItemKind::HttpBlock(hb) => {
            assert_eq!(hb.method, HttpMethod::Delete);
        }
        other => panic!("expected HttpBlock, got {:?}", other),
    }
}

#[test]
fn multiple_http_blocks() {
    let result = parse_ok(
        "get { return 1; }
         post { return 2; }
         delete { return 3; }",
    );
    assert_eq!(result.program.items.len(), 3);
    assert!(matches!(
        &result.program.items[0].kind,
        ItemKind::HttpBlock(hb) if hb.method == HttpMethod::Get
    ));
    assert!(matches!(
        &result.program.items[1].kind,
        ItemKind::HttpBlock(hb) if hb.method == HttpMethod::Post
    ));
    assert!(matches!(
        &result.program.items[2].kind,
        ItemKind::HttpBlock(hb) if hb.method == HttpMethod::Delete
    ));
}

#[test]
fn init_block() {
    let item = parse_single_item("init { foo(); }");
    match &item.kind {
        ItemKind::InitBlock(ib) => {
            assert_eq!(ib.body.stmts.len(), 1);
        }
        other => panic!("expected InitBlock, got {:?}", other),
    }
}

#[test]
fn error_handler() {
    let item = parse_single_item("error(err) { return 500; }");
    match &item.kind {
        ItemKind::ErrorHandler(eh) => {
            assert_eq!(eh.param.name, "err");
            assert_eq!(eh.body.stmts.len(), 1);
        }
        other => panic!("expected ErrorHandler, got {:?}", other),
    }
}

#[test]
fn http_block_with_complex_body() {
    let result = parse_ok(
        "get {
            const user = self.params;
            self.response.status(200);
            return user;
        }",
    );
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::HttpBlock(hb) => {
            assert_eq!(hb.method, HttpMethod::Get);
            assert_eq!(hb.body.stmts.len(), 3);
        }
        other => panic!("expected HttpBlock, got {:?}", other),
    }
}
