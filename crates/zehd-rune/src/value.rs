use std::fmt;

// ── Value ──────────────────────────────────────────────────────

/// Runtime value stored in the constant pool or on the VM stack.
///
/// This is shared with the future VM crate. Values here represent
/// compile-time constants; the VM will extend this with heap-allocated
/// variants for runtime values.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    None,
    Unit,
    List(Vec<Value>),
    Object(Vec<(String, Value)>),
    /// Index into CompiledModule.functions.
    Function(u16),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::None => write!(f, "None"),
            Value::Unit => write!(f, "()"),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Value::Object(fields) => {
                write!(f, "{{ ")?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, " }}")
            }
            Value::Function(idx) => write!(f, "<fn:{idx}>"),
        }
    }
}
