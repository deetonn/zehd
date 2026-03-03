use zehd_tome::Span;

// ── Node Identity ───────────────────────────────────────────────

/// Unique identifier for AST nodes. Used by the type checker to associate
/// types and resolutions with specific nodes via side tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

// ── Shared Types ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

// ── Program & Items ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: NodeId,
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Import(ImportItem),
    TypeDef(TypeDef),
    EnumDef(EnumDef),
    Function(Function),
    VarDecl(VarDecl),
    HttpBlock(HttpBlock),
    InitBlock(InitBlock),
    ErrorHandler(ErrorHandler),
    ExprStmt(ExprStmt),
}

// ── Declarations ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ImportItem {
    pub names: Vec<ImportName>,
    pub path: ImportPath,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportName {
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportPath {
    pub segments: Vec<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: Ident,
    pub type_params: Vec<Ident>,
    pub fields: Vec<TypeField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeField {
    pub attributes: Vec<Attribute>,
    pub name: Ident,
    pub ty: TypeAnnotation,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: Ident,
    pub type_params: Vec<Ident>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: Ident,
    pub payload: Option<TypeAnnotation>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Ident,
    pub ty: Option<TypeAnnotation>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub mutable: bool, // true = let, false = const
    pub name: Ident,
    pub ty: Option<TypeAnnotation>,
    pub initializer: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpBlock {
    pub method: HttpMethod,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InitBlock {
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorHandler {
    pub param: Ident,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub path: Vec<Ident>, // [validate, min] for `validate.min`
    pub args: Vec<Expr>,
    pub span: Span,
}

// ── Type Annotations ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAnnotation {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Named(Ident),
    Generic {
        name: Ident,
        args: Vec<TypeAnnotation>,
    },
    Function {
        params: Vec<TypeAnnotation>,
        return_type: Box<TypeAnnotation>,
    },
}

// ── Statements ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    VarDecl(VarDecl),
    ExprStmt(ExprStmt),
    Return(ReturnStmt),
    Break,
    Continue,
    For(ForStmt),
    While(WhileStmt),
    Assignment(Assignment),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub binding: Ident,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub target: Expr,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail_expr: Option<Box<Expr>>,
    pub span: Span,
}

// ── Expressions ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    TimeLiteral(u64),
    BoolLiteral(bool),
    NoneLiteral,

    // Enum constructors: Some(x), Ok(x), Err(x)
    EnumConstructor { name: Ident, arg: Box<Expr> },

    // Identifiers
    Ident(Ident),
    SelfExpr,

    // Operators
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },

    // Postfix
    Try(Box<Expr>),
    FieldAccess {
        object: Box<Expr>,
        field: Ident,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        type_args: Vec<TypeAnnotation>,
        args: Vec<Expr>,
    },

    // Compound
    If {
        condition: Box<Expr>,
        then_block: Block,
        else_block: Option<ElseBranch>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    ArrowFunction {
        params: Vec<Param>,
        return_type: Option<TypeAnnotation>,
        body: ArrowBody,
    },
    ObjectLiteral {
        fields: Vec<ObjectField>,
    },
    ListLiteral {
        elements: Vec<Expr>,
    },
    InterpolatedString {
        parts: Vec<InterpolatedPart>,
    },
    Block(Block),
    Grouped(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElseBranch {
    ElseBlock(Block),
    ElseIf(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrowBody {
    Expr(Box<Expr>),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectField {
    pub key: Ident,
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedPart {
    Literal(String, Span),
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

// ── Patterns ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternKind {
    Wildcard,
    Binding(Ident),
    Literal(LiteralPattern),
    EnumVariant {
        path: Vec<Ident>,
        binding: Option<Box<Pattern>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralPattern {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}
