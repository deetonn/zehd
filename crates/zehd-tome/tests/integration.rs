mod helpers;
use helpers::*;
use zehd_tome::TokenKind;

#[test]
fn example_z_get_block() {
    let source = r#"
import { proxy } from std;
import { Response } from std::types;

get {
    const res: Response = self.response;

    res.headers.set("x-server-location", "lhr1:233");

    return {
        name: "Zehd",
        version: "1.2.3",
    };
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    // Just verify it lexes fully and has the expected structure
    assert!(k.contains(&TokenKind::Import));
    assert!(k.contains(&TokenKind::From));
    assert!(k.contains(&TokenKind::Get));
    assert!(k.contains(&TokenKind::Const));
    assert!(k.contains(&TokenKind::SelfKw));
    assert!(k.contains(&TokenKind::Return));
    assert!(k.contains(&TokenKind::String));
    assert!(k.contains(&TokenKind::ColonColon));
}

#[test]
fn example_z_init_block() {
    let source = r#"
import { use, rateLimit } from std;

use(rateLimit("rollingWindow", 60s));
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Import));
    assert!(k.contains(&TokenKind::From));
    assert!(k.contains(&TokenKind::TimeLiteral(60_000)));
}

#[test]
fn route_handler_with_params() {
    let source = r#"
type UserParams {
    id: string;
}

get {
    const params: UserParams = self.params.parse();
    const user = db.users.find(params.id);

    if !user {
        self.response.status(404);
        return { error: "User not found" };
    }

    return user;
}

post {
    const body: CreateUser = self.request.json();
    const user = db.users.create(body);
    self.response.status(201);
    return user;
}

delete {
    const params: UserParams = self.params.parse();
    db.users.delete(params.id);
    self.response.status(204);
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Type));
    assert!(k.contains(&TokenKind::Get));
    assert!(k.contains(&TokenKind::Post));
    assert!(k.contains(&TokenKind::Delete));
    assert!(k.contains(&TokenKind::If));
    assert!(k.contains(&TokenKind::Bang));
    assert!(k.contains(&TokenKind::SelfKw));
}

#[test]
fn init_z_with_cors_and_auth() {
    let source = r#"
import { use, cors, rateLimit } from std;
import { auth } from std::auth;

init {
    use(cors({ origins: ["https://myapp.com"] }));
    use(rateLimit("rollingWindow", 60s));
    use(auth.bearer());
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Init));
    assert!(k.contains(&TokenKind::ColonColon));
    assert!(k.contains(&TokenKind::TimeLiteral(60_000)));
}

#[test]
fn type_definition_with_attributes() {
    let source = r#"
import { validate } from std::validation;

type CreateUser {
    #[validate.min(1)]
    #[validate.max(100)]
    name: string;

    #[validate.range(18, 150)]
    age: int;

    #[validate.email]
    email: string;
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Hash));
    assert!(k.contains(&TokenKind::LeftBracket));
    assert!(k.contains(&TokenKind::RightBracket));
    assert!(k.contains(&TokenKind::Type));
}

#[test]
fn error_handler() {
    let source = r#"
error(err) {
    match err {
        UserError.NotFound(msg) => {
            self.response.status(404);
            return { error: msg };
        }
        _ => {
            self.response.status(500);
            return { error: "Internal server error" };
        }
    }
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Error));
    assert!(k.contains(&TokenKind::Match));
    assert!(k.contains(&TokenKind::FatArrow));
    assert!(k.contains(&TokenKind::Underscore));
}

#[test]
fn provide_inject_pattern() {
    let source = r#"
import { provide } from std;
import { DbPool, createPool } from lib;

const pool = createPool({ url: env("DATABASE_URL") });
provide<DbPool>(pool);
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Const));
    assert!(k.contains(&TokenKind::Lt));
    assert!(k.contains(&TokenKind::Gt));
}

#[test]
fn arrow_function() {
    let source = r#"const double = (x: int) => x * 2;"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::FatArrow));
    assert!(k.contains(&TokenKind::Star));
    assert!(k.contains(&TokenKind::Integer(2)));
}

#[test]
fn enum_definition() {
    let source = r#"
enum UserError {
    NotFound(string),
    Unauthorized,
    ValidationFailed(list<string>),
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Enum));
    assert!(k.contains(&TokenKind::Comma));
    assert!(k.contains(&TokenKind::Lt));
    assert!(k.contains(&TokenKind::Gt));
}

#[test]
fn string_interpolation_in_context() {
    let source = r#"
fn greet(name: string): string {
    return $"Hello, {name}!";
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Fn));
    assert!(k.contains(&TokenKind::InterpolatedStringStart));
    assert!(k.contains(&TokenKind::InterpolatedStringEnd));
}

#[test]
fn match_expression() {
    let source = r#"
match db.users.find(id) {
    Ok(user) => return user;
    Err(UserError.NotFound(msg)) => {
        self.response.status(404);
        return { error: msg };
    }
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Match));
    assert!(k.contains(&TokenKind::Ok));
    assert!(k.contains(&TokenKind::Err));
    assert!(k.contains(&TokenKind::FatArrow));
}

#[test]
fn for_while_loops() {
    let source = r#"
for item in items {
    log.info(item);
}

let attempts = 0;
while attempts < 3 {
    attempts = attempts + 1;
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::For));
    assert!(k.contains(&TokenKind::In));
    assert!(k.contains(&TokenKind::While));
    assert!(k.contains(&TokenKind::Let));
}

#[test]
fn if_expression() {
    let source = r#"const label = if count > 10 { "many" } else { "few" };"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::If));
    assert!(k.contains(&TokenKind::Else));
    assert!(k.contains(&TokenKind::Gt));
}

#[test]
fn question_mark_operator() {
    let source = r#"
get {
    const user = db.users.find(id)?;
    const posts = db.posts.by_user(user.id)?;
    return { user, posts };
}
"#;
    let result = lex_ok(source);
    assert!(result.is_ok());

    let k = kinds(source);
    assert!(k.contains(&TokenKind::Question));
}
