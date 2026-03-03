mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn route_file_with_imports_and_get() {
    let result = parse_ok(
        "import { proxy } from std;

         get {
             const res = self.response;
             res.headers.set(\"x-server-location\", \"lhr1:233\");
             return {
                 name: \"Zehd\",
                 version: \"1.2.3\",
             };
         }",
    );
    assert_eq!(result.program.items.len(), 2);
    assert!(matches!(&result.program.items[0].kind, ItemKind::Import(_)));
    assert!(matches!(
        &result.program.items[1].kind,
        ItemKind::HttpBlock(hb) if hb.method == HttpMethod::Get
    ));
}

#[test]
fn init_file_with_middleware() {
    let result = parse_ok(
        "import { use, rateLimit } from std;
         use(rateLimit(\"rollingWindow\", 60s));",
    );
    assert_eq!(result.program.items.len(), 2);
    assert!(matches!(&result.program.items[0].kind, ItemKind::Import(_)));
    assert!(matches!(
        &result.program.items[1].kind,
        ItemKind::ExprStmt(_)
    ));
}

#[test]
fn type_with_validation_attributes() {
    let result = parse_ok(
        "import { validate } from std::validation;

         type CreateUser {
             #[validate.min(1)]
             #[validate.max(100)]
             name: string;
             #[validate.range(18, 150)]
             age: int;
             #[validate.email]
             email: string;
         }",
    );
    assert_eq!(result.program.items.len(), 2);
    match &result.program.items[1].kind {
        ItemKind::TypeDef(td) => {
            assert_eq!(td.name.name, "CreateUser");
            assert_eq!(td.fields.len(), 3);
        }
        other => panic!("expected TypeDef, got {:?}", other),
    }
}

#[test]
fn route_with_type_and_handlers() {
    let result = parse_ok(
        "type UserParams {
             id: string;
         }

         get {
             const params = self.params;
             return params;
         }

         post {
             const body = self.request;
             return body;
         }

         delete {
             return 204;
         }",
    );
    assert_eq!(result.program.items.len(), 4);
    assert!(matches!(&result.program.items[0].kind, ItemKind::TypeDef(_)));
    assert!(matches!(&result.program.items[1].kind, ItemKind::HttpBlock(_)));
    assert!(matches!(&result.program.items[2].kind, ItemKind::HttpBlock(_)));
    assert!(matches!(&result.program.items[3].kind, ItemKind::HttpBlock(_)));
}

#[test]
fn error_handler_with_match() {
    let result = parse_ok(
        "error(err) {
             match err {
                 UserError.NotFound(msg) => {
                     self.response.status(404);
                     return { error: msg };
                 }
                 _ => {
                     self.response.status(500);
                     return { error: \"Internal server error\" };
                 }
             }
         }",
    );
    assert_eq!(result.program.items.len(), 1);
    assert!(matches!(
        &result.program.items[0].kind,
        ItemKind::ErrorHandler(_)
    ));
}

#[test]
fn enum_definition_and_match() {
    let result = parse_ok(
        "enum UserError {
             NotFound(string),
             Unauthorized,
         }

         fn handle(err: UserError): int {
             match err {
                 UserError.NotFound(msg) => 404,
                 UserError.Unauthorized => 403,
             }
         }",
    );
    assert_eq!(result.program.items.len(), 2);
    assert!(matches!(&result.program.items[0].kind, ItemKind::EnumDef(_)));
    assert!(matches!(
        &result.program.items[1].kind,
        ItemKind::Function(_)
    ));
}

#[test]
fn function_with_if_expression() {
    let result = parse_ok(
        "fn classify(count: int): string {
             const label = if count > 10 { \"many\" } else { \"few\" };
             return label;
         }",
    );
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    assert_eq!(func.body.stmts.len(), 2);
}

#[test]
fn for_loop_over_collection() {
    let result = parse_ok(
        "fn process(items: list) {
             for item in items {
                 log(item);
             }
         }",
    );
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    assert_eq!(func.body.stmts.len(), 1);
    assert!(matches!(func.body.stmts[0].kind, StmtKind::For(_)));
}

#[test]
fn di_provide_inject() {
    let result = parse_ok(
        "import { provide } from std;
         const pool = createPool();
         provide(pool);",
    );
    assert_eq!(result.program.items.len(), 3);
}

#[test]
fn arrow_function_expression_body() {
    let result = parse_ok("const double = (x: int) => x * 2;");
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.initializer.as_ref().unwrap().kind {
                ExprKind::ArrowFunction { params, body, .. } => {
                    assert_eq!(params.len(), 1);
                    assert!(matches!(body, ArrowBody::Expr(_)));
                }
                other => panic!("expected ArrowFunction, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn arrow_function_block_body() {
    let result = parse_ok(
        "const transform = (req) => {
             req.headers.set(\"Auth\", \"key\");
         };",
    );
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.initializer.as_ref().unwrap().kind {
                ExprKind::ArrowFunction { params, body, .. } => {
                    assert_eq!(params.len(), 1);
                    assert!(matches!(body, ArrowBody::Block(_)));
                }
                other => panic!("expected ArrowFunction, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn object_literal_shorthand() {
    let result = parse_ok("const obj = { name, version };");
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.initializer.as_ref().unwrap().kind {
                ExprKind::ObjectLiteral { fields } => {
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].key.name, "name");
                    assert!(fields[0].value.is_none());
                    assert_eq!(fields[1].key.name, "version");
                    assert!(fields[1].value.is_none());
                }
                other => panic!("expected ObjectLiteral, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn object_literal_with_values() {
    let result = parse_ok(
        "const obj = {
             name: \"Zehd\",
             version: \"1.2.3\",
         };",
    );
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.initializer.as_ref().unwrap().kind {
                ExprKind::ObjectLiteral { fields } => {
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].key.name, "name");
                    assert!(fields[0].value.is_some());
                }
                other => panic!("expected ObjectLiteral, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn list_literal() {
    let result = parse_ok("const items = [1, 2, 3];");
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.initializer.as_ref().unwrap().kind {
                ExprKind::ListLiteral { elements } => {
                    assert_eq!(elements.len(), 3);
                }
                other => panic!("expected ListLiteral, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn try_operator_in_chain() {
    let result = parse_ok(
        "get {
             const user = db.users.find(id)?;
             const posts = db.posts.by_user(user.id)?;
             return { user, posts };
         }",
    );
    let hb = match &result.program.items[0].kind {
        ItemKind::HttpBlock(hb) => hb,
        other => panic!("expected HttpBlock, got {:?}", other),
    };
    assert_eq!(hb.body.stmts.len(), 3);
}

#[test]
fn while_loop_with_counter() {
    let result = parse_ok(
        "fn retry() {
             let attempts = 0;
             while attempts < 3 {
                 attempts = attempts + 1;
             }
         }",
    );
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    assert_eq!(func.body.stmts.len(), 2);
}

#[test]
fn if_else_chain() {
    let result = parse_ok(
        "fn classify(x: int): string {
             if x > 100 {
                 return \"big\";
             } else if x > 10 {
                 return \"medium\";
             } else {
                 return \"small\";
             }
         }",
    );
    assert_eq!(result.program.items.len(), 1);
}

#[test]
fn empty_program() {
    let result = parse_ok("");
    assert!(result.program.items.is_empty());
}

#[test]
fn full_route_example() {
    // Full example from DESIGN.md: routes/users/[id].z
    let result = parse_ok(
        "type UserParams {
             id: string;
         }

         get {
             const params = self.params;
             const user = db.users.find(params.id);

             if !user {
                 self.response.status(404);
                 return { error: \"User not found\" };
             }

             return user;
         }

         post {
             const params = self.params;
             const body = self.request;
             const user = db.users.create(body);
             self.response.status(201);
             return user;
         }

         delete {
             const params = self.params;
             db.users.delete(params.id);
             self.response.status(204);
         }",
    );
    assert_eq!(result.program.items.len(), 4);
}
