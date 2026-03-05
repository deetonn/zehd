use crate::types::Type;

// TODO: Add list.map(fn), list.filter(fn), list.find(fn), list.flat_map(fn)
//       — requires VM re-entrancy to call user functions from method dispatch.
// TODO: list.push(item) currently returns a new list (immutable).
//       Switch to in-place mutation once reference semantics are implemented.

// ── Method IDs ──────────────────────────────────────────────────

// String methods (0–11)
pub const STRING_LENGTH: u16 = 0;
pub const STRING_CONTAINS: u16 = 1;
pub const STRING_STARTS_WITH: u16 = 2;
pub const STRING_ENDS_WITH: u16 = 3;
pub const STRING_TRIM: u16 = 4;
pub const STRING_TO_UPPER: u16 = 5;
pub const STRING_TO_LOWER: u16 = 6;
pub const STRING_SPLIT: u16 = 7;
pub const STRING_REPLACE: u16 = 8;
pub const STRING_SUBSTRING: u16 = 9;
pub const STRING_INDEX_OF: u16 = 10;
pub const STRING_CHAR_AT: u16 = 11;

// List methods (12–17)
pub const LIST_LENGTH: u16 = 12;
pub const LIST_PUSH: u16 = 13;
pub const LIST_CONTAINS: u16 = 14;
pub const LIST_JOIN: u16 = 15;
pub const LIST_REVERSE: u16 = 16;
pub const LIST_SLICE: u16 = 17;

// Int methods (18, 20, 22)
pub const INT_TO_STRING: u16 = 18;
pub const INT_ABS: u16 = 20;
pub const INT_TO_FLOAT: u16 = 22;

// Float methods (19, 21, 23–25)
pub const FLOAT_TO_STRING: u16 = 19;
pub const FLOAT_ABS: u16 = 21;
pub const FLOAT_FLOOR: u16 = 23;
pub const FLOAT_CEIL: u16 = 24;
pub const FLOAT_ROUND: u16 = 25;

// ── Method Signature ────────────────────────────────────────────

pub struct MethodSig {
    pub method_id: u16,
    pub params: Vec<Type>,
    pub return_type: Type,
}

// ── Resolve ─────────────────────────────────────────────────────

/// Method name + signature pair for listing all methods on a type.
pub struct MethodEntry {
    pub name: &'static str,
    pub sig: MethodSig,
}

/// List all built-in methods available on a given type.
/// Used by the LSP for completions and hover.
pub fn builtin_methods_for_type(receiver_ty: &Type) -> Vec<MethodEntry> {
    let names: &[&str] = match receiver_ty {
        Type::String => &[
            "length", "contains", "starts_with", "ends_with", "trim",
            "to_upper", "to_lower", "split", "replace", "substring",
            "index_of", "char_at",
        ],
        Type::List(_) => &[
            "length", "push", "contains", "join", "reverse", "slice",
        ],
        Type::Int => &["to_string", "abs", "to_float"],
        Type::Float => &["to_string", "abs", "floor", "ceil", "round"],
        _ => return vec![],
    };
    names
        .iter()
        .filter_map(|name| {
            resolve_builtin_method(receiver_ty, name)
                .map(|sig| MethodEntry { name, sig })
        })
        .collect()
}

/// Resolve a built-in method by receiver type and method name.
/// Returns `None` if no such method exists for the given type.
pub fn resolve_builtin_method(receiver_ty: &Type, method_name: &str) -> Option<MethodSig> {
    match receiver_ty {
        Type::String => resolve_string_method(method_name),
        Type::List(elem) => resolve_list_method(method_name, elem),
        Type::Int => resolve_int_method(method_name),
        Type::Float => resolve_float_method(method_name),
        _ => None,
    }
}

fn resolve_string_method(name: &str) -> Option<MethodSig> {
    match name {
        "length" => Some(MethodSig {
            method_id: STRING_LENGTH,
            params: vec![],
            return_type: Type::Int,
        }),
        "contains" => Some(MethodSig {
            method_id: STRING_CONTAINS,
            params: vec![Type::String],
            return_type: Type::Bool,
        }),
        "starts_with" => Some(MethodSig {
            method_id: STRING_STARTS_WITH,
            params: vec![Type::String],
            return_type: Type::Bool,
        }),
        "ends_with" => Some(MethodSig {
            method_id: STRING_ENDS_WITH,
            params: vec![Type::String],
            return_type: Type::Bool,
        }),
        "trim" => Some(MethodSig {
            method_id: STRING_TRIM,
            params: vec![],
            return_type: Type::String,
        }),
        "to_upper" => Some(MethodSig {
            method_id: STRING_TO_UPPER,
            params: vec![],
            return_type: Type::String,
        }),
        "to_lower" => Some(MethodSig {
            method_id: STRING_TO_LOWER,
            params: vec![],
            return_type: Type::String,
        }),
        "split" => Some(MethodSig {
            method_id: STRING_SPLIT,
            params: vec![Type::String],
            return_type: Type::List(Box::new(Type::String)),
        }),
        "replace" => Some(MethodSig {
            method_id: STRING_REPLACE,
            params: vec![Type::String, Type::String],
            return_type: Type::String,
        }),
        "substring" => Some(MethodSig {
            method_id: STRING_SUBSTRING,
            params: vec![Type::Int, Type::Int],
            return_type: Type::String,
        }),
        "index_of" => Some(MethodSig {
            method_id: STRING_INDEX_OF,
            params: vec![Type::String],
            return_type: Type::Option(Box::new(Type::Int)),
        }),
        "char_at" => Some(MethodSig {
            method_id: STRING_CHAR_AT,
            params: vec![Type::Int],
            return_type: Type::Option(Box::new(Type::String)),
        }),
        _ => None,
    }
}

fn resolve_list_method(name: &str, elem: &Type) -> Option<MethodSig> {
    match name {
        "length" => Some(MethodSig {
            method_id: LIST_LENGTH,
            params: vec![],
            return_type: Type::Int,
        }),
        "push" => Some(MethodSig {
            method_id: LIST_PUSH,
            params: vec![elem.clone()],
            return_type: Type::List(Box::new(elem.clone())),
        }),
        "contains" => Some(MethodSig {
            method_id: LIST_CONTAINS,
            params: vec![elem.clone()],
            return_type: Type::Bool,
        }),
        "join" => Some(MethodSig {
            method_id: LIST_JOIN,
            params: vec![Type::String],
            return_type: Type::String,
        }),
        "reverse" => Some(MethodSig {
            method_id: LIST_REVERSE,
            params: vec![],
            return_type: Type::List(Box::new(elem.clone())),
        }),
        "slice" => Some(MethodSig {
            method_id: LIST_SLICE,
            params: vec![Type::Int, Type::Int],
            return_type: Type::List(Box::new(elem.clone())),
        }),
        _ => None,
    }
}

fn resolve_int_method(name: &str) -> Option<MethodSig> {
    match name {
        "to_string" => Some(MethodSig {
            method_id: INT_TO_STRING,
            params: vec![],
            return_type: Type::String,
        }),
        "abs" => Some(MethodSig {
            method_id: INT_ABS,
            params: vec![],
            return_type: Type::Int,
        }),
        "to_float" => Some(MethodSig {
            method_id: INT_TO_FLOAT,
            params: vec![],
            return_type: Type::Float,
        }),
        _ => None,
    }
}

fn resolve_float_method(name: &str) -> Option<MethodSig> {
    match name {
        "to_string" => Some(MethodSig {
            method_id: FLOAT_TO_STRING,
            params: vec![],
            return_type: Type::String,
        }),
        "abs" => Some(MethodSig {
            method_id: FLOAT_ABS,
            params: vec![],
            return_type: Type::Float,
        }),
        "floor" => Some(MethodSig {
            method_id: FLOAT_FLOOR,
            params: vec![],
            return_type: Type::Int,
        }),
        "ceil" => Some(MethodSig {
            method_id: FLOAT_CEIL,
            params: vec![],
            return_type: Type::Int,
        }),
        "round" => Some(MethodSig {
            method_id: FLOAT_ROUND,
            params: vec![],
            return_type: Type::Int,
        }),
        _ => None,
    }
}
