use std::fmt;

/// Index into the type side table or inference context.
pub type TypeId = u32;

/// Type variable identifier — index into the union-find structure.
pub type TypeVar = u32;

/// The internal type representation used by the checker.
///
/// This is distinct from the parser's `TypeAnnotation`/`TypeKind` — it represents
/// resolved, fully-checked types with inference variables, structural comparison,
/// and resolved references.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // ── Primitives ──────────────────────────────────────────────
    Int,
    Float,
    String,
    Bool,
    /// Time literals resolve to Int (milliseconds) but track origin for errors.
    Time,
    /// Absence of value — empty return, void blocks, statement results.
    Unit,

    // ── Constructed ─────────────────────────────────────────────
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),

    // ── User-defined ────────────────────────────────────────────
    Struct(StructType),
    Enum(EnumType),

    // ── Functions ───────────────────────────────────────────────
    Function(FunctionType),

    // ── Inference ───────────────────────────────────────────────
    /// Unresolved type variable — placeholder during inference.
    Var(TypeVar),
    /// Diverging type — produced by `return`, `break` in all branches.
    Never,

    // ── Error sentinel ──────────────────────────────────────────
    /// Produced on type errors to prevent cascading diagnostics.
    Error,
}

/// A struct type with named fields.
#[derive(Debug, Clone, PartialEq)]
pub struct StructType {
    /// `None` for anonymous/structural types (object literals).
    pub name: Option<String>,
    /// Fields ordered for display; compared as a set for structural typing.
    pub fields: Vec<(String, Type)>,
    /// Resolved generic type arguments.
    pub type_params: Vec<Type>,
}

/// An enum type with variants.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<EnumVariantType>,
    pub type_params: Vec<Type>,
}

/// A single enum variant with optional payload.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantType {
    pub name: String,
    pub payload: Option<Type>,
}

/// A function type: params → return.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
}

// ── Structural Compatibility ────────────────────────────────────

impl Type {
    /// Check if `source` is structurally compatible with `target`.
    ///
    /// For struct types, every field in `target` must exist in `source`
    /// with a compatible type (width subtyping). Name is irrelevant.
    pub fn is_compatible(source: &Type, target: &Type) -> bool {
        match (source, target) {
            // Error type is compatible with everything (prevents cascading).
            (Type::Error, _) | (_, Type::Error) => true,

            // Never is compatible with any type (diverging code).
            (Type::Never, _) => true,

            // Primitives match exactly.
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::String, Type::String) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Unit, Type::Unit) => true,

            // Time unifies with Int.
            (Type::Time, Type::Int) | (Type::Int, Type::Time) | (Type::Time, Type::Time) => true,

            // Constructed types — check inner types.
            (Type::Option(a), Type::Option(b)) => Type::is_compatible(a, b),
            (Type::Result(a1, a2), Type::Result(b1, b2)) => {
                Type::is_compatible(a1, b1) && Type::is_compatible(a2, b2)
            }
            (Type::List(a), Type::List(b)) => Type::is_compatible(a, b),
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                Type::is_compatible(k1, k2) && Type::is_compatible(v1, v2)
            }

            // Struct types — structural comparison.
            (Type::Struct(src), Type::Struct(tgt)) => is_structurally_compatible(src, tgt),

            // Enum types — must be the same enum (by name + variants).
            (Type::Enum(a), Type::Enum(b)) => a.name == b.name,

            // Function types — contravariant params, covariant return.
            (Type::Function(a), Type::Function(b)) => {
                a.params.len() == b.params.len()
                    && a.params
                        .iter()
                        .zip(&b.params)
                        .all(|(ap, bp)| Type::is_compatible(bp, ap))
                    && Type::is_compatible(&a.return_type, &b.return_type)
            }

            _ => false,
        }
    }

    /// Returns true if this type is the error sentinel.
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }

    /// Returns true if this type is a type variable (unresolved).
    pub fn is_var(&self) -> bool {
        matches!(self, Type::Var(_))
    }
}

/// Structural compatibility check for struct types.
///
/// Every field in `target` must exist in `source` with a compatible type.
/// Width subtyping: source may have extra fields.
fn is_structurally_compatible(source: &StructType, target: &StructType) -> bool {
    target.fields.iter().all(|(name, ty)| {
        source
            .fields
            .iter()
            .any(|(sn, st)| sn == name && Type::is_compatible(st, ty))
    })
}

// ── Display ─────────────────────────────────────────────────────

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Time => write!(f, "time"),
            Type::Unit => write!(f, "()"),
            Type::Option(inner) => write!(f, "Option<{inner}>"),
            Type::Result(ok, err) => write!(f, "Result<{ok}, {err}>"),
            Type::List(elem) => write!(f, "List<{elem}>"),
            Type::Map(k, v) => write!(f, "Map<{k}, {v}>"),
            Type::Struct(s) => {
                if let Some(name) = &s.name {
                    write!(f, "{name}")
                } else {
                    write!(f, "{{ ")?;
                    for (i, (name, ty)) in s.fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{name}: {ty}")?;
                    }
                    write!(f, " }}")
                }
            }
            Type::Enum(e) => write!(f, "{}", e.name),
            Type::Function(ft) => {
                write!(f, "(")?;
                for (i, p) in ft.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ") => {}", ft.return_type)
            }
            Type::Var(v) => write!(f, "?{v}"),
            Type::Never => write!(f, "never"),
            Type::Error => write!(f, "<error>"),
        }
    }
}
