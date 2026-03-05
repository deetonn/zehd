use crate::error::{RuntimeError, RuntimeErrorCode};
use zehd_rune::value::Value;

// TODO: Add list.map(fn), list.filter(fn), list.find(fn), list.flat_map(fn)
//       These require VM re-entrancy to call user functions from method dispatch.
// TODO: list.push(item) currently returns a new list (immutable).
//       Switch to in-place mutation once reference semantics are implemented.

/// Dispatch a built-in method call by method_id.
///
/// The `receiver` is the value the method is called on.
/// `args` are the method arguments (NOT including the receiver).
pub fn dispatch_method(
    method_id: u16,
    receiver: Value,
    args: &[Value],
) -> Result<Value, RuntimeError> {
    match method_id {
        // ── String methods (0–11) ───────────────────────────
        0 => {
            // string.length
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            Ok(Value::Int(s.len() as i64))
        }
        1 => {
            // string.contains(needle)
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            let Value::String(needle) = &args[0] else {
                return Err(arg_type_error("string", &args[0]));
            };
            Ok(Value::Bool(s.contains(needle.as_str())))
        }
        2 => {
            // string.starts_with(prefix)
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            let Value::String(prefix) = &args[0] else {
                return Err(arg_type_error("string", &args[0]));
            };
            Ok(Value::Bool(s.starts_with(prefix.as_str())))
        }
        3 => {
            // string.ends_with(suffix)
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            let Value::String(suffix) = &args[0] else {
                return Err(arg_type_error("string", &args[0]));
            };
            Ok(Value::Bool(s.ends_with(suffix.as_str())))
        }
        4 => {
            // string.trim
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            Ok(Value::String(s.trim().to_string()))
        }
        5 => {
            // string.to_upper
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            Ok(Value::String(s.to_uppercase()))
        }
        6 => {
            // string.to_lower
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            Ok(Value::String(s.to_lowercase()))
        }
        7 => {
            // string.split(delimiter)
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            let Value::String(delim) = &args[0] else {
                return Err(arg_type_error("string", &args[0]));
            };
            let parts: Vec<Value> = s.split(delim.as_str()).map(|p| Value::String(p.to_string())).collect();
            Ok(Value::List(parts))
        }
        8 => {
            // string.replace(from, to)
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            let Value::String(from) = &args[0] else {
                return Err(arg_type_error("string", &args[0]));
            };
            let Value::String(to) = &args[1] else {
                return Err(arg_type_error("string", &args[1]));
            };
            Ok(Value::String(s.replace(from.as_str(), to.as_str())))
        }
        9 => {
            // string.substring(start, end) → Result<string, string>
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            let Value::Int(start) = &args[0] else {
                return Err(arg_type_error("int", &args[0]));
            };
            let Value::Int(end) = &args[1] else {
                return Err(arg_type_error("int", &args[1]));
            };
            let start = *start;
            let end = *end;
            let char_len = s.chars().count() as i64;
            if start < 0 || end < 0 || start > char_len || end > char_len || start > end {
                Ok(result_err(format!(
                    "substring indices [{start}, {end}) out of bounds for string of length {char_len}"
                )))
            } else {
                let result: String = s.chars().skip(start as usize).take((end - start) as usize).collect();
                Ok(result_ok(Value::String(result)))
            }
        }
        10 => {
            // string.index_of(needle) → Option<int>
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            let Value::String(needle) = &args[0] else {
                return Err(arg_type_error("string", &args[0]));
            };
            match s.find(needle.as_str()) {
                Some(idx) => Ok(Value::Enum {
                    type_idx: 0xFFFE,
                    variant_idx: 0,
                    payload: Some(Box::new(Value::Int(idx as i64))),
                }),
                None => Ok(Value::Enum {
                    type_idx: 0xFFFE,
                    variant_idx: 1,
                    payload: None,
                }),
            }
        }
        11 => {
            // string.char_at(index) → Option<string>
            let Value::String(s) = &receiver else {
                return Err(type_error("string", &receiver));
            };
            let Value::Int(idx) = &args[0] else {
                return Err(arg_type_error("int", &args[0]));
            };
            let idx = *idx;
            if idx < 0 {
                return Ok(Value::Enum {
                    type_idx: 0xFFFE,
                    variant_idx: 1,
                    payload: None,
                });
            }
            match s.chars().nth(idx as usize) {
                Some(ch) => Ok(Value::Enum {
                    type_idx: 0xFFFE,
                    variant_idx: 0,
                    payload: Some(Box::new(Value::String(ch.to_string()))),
                }),
                None => Ok(Value::Enum {
                    type_idx: 0xFFFE,
                    variant_idx: 1,
                    payload: None,
                }),
            }
        }

        // ── List methods (12–17) ────────────────────────────
        12 => {
            // list.length
            let Value::List(items) = &receiver else {
                return Err(type_error("list", &receiver));
            };
            Ok(Value::Int(items.len() as i64))
        }
        13 => {
            // list.push(item) — returns new list (immutable)
            let Value::List(mut items) = receiver else {
                return Err(type_error("list", &args[0]));
            };
            items.push(args[0].clone());
            Ok(Value::List(items))
        }
        14 => {
            // list.contains(item)
            let Value::List(items) = &receiver else {
                return Err(type_error("list", &receiver));
            };
            Ok(Value::Bool(items.contains(&args[0])))
        }
        15 => {
            // list.join(separator)
            let Value::List(items) = &receiver else {
                return Err(type_error("list", &receiver));
            };
            let Value::String(sep) = &args[0] else {
                return Err(arg_type_error("string", &args[0]));
            };
            let parts: Vec<String> = items.iter().map(value_to_string).collect();
            Ok(Value::String(parts.join(sep.as_str())))
        }
        16 => {
            // list.reverse
            let Value::List(mut items) = receiver else {
                return Err(type_error("list", &args[0]));
            };
            items.reverse();
            Ok(Value::List(items))
        }
        17 => {
            // list.slice(start, end) → Result<List<T>, string>
            let Value::List(items) = &receiver else {
                return Err(type_error("list", &receiver));
            };
            let Value::Int(start) = &args[0] else {
                return Err(arg_type_error("int", &args[0]));
            };
            let Value::Int(end) = &args[1] else {
                return Err(arg_type_error("int", &args[1]));
            };
            let start = *start;
            let end = *end;
            let len = items.len() as i64;
            if start < 0 || end < 0 || start > len || end > len || start > end {
                Ok(result_err(format!(
                    "slice indices [{start}, {end}) out of bounds for list of length {len}"
                )))
            } else {
                Ok(result_ok(Value::List(items[start as usize..end as usize].to_vec())))
            }
        }

        // ── Int methods (18, 20, 22) ────────────────────────
        18 => {
            // int.to_string
            let Value::Int(n) = &receiver else {
                return Err(type_error("int", &receiver));
            };
            Ok(Value::String(n.to_string()))
        }
        20 => {
            // int.abs
            let Value::Int(n) = &receiver else {
                return Err(type_error("int", &receiver));
            };
            Ok(Value::Int(n.abs()))
        }
        22 => {
            // int.to_float
            let Value::Int(n) = &receiver else {
                return Err(type_error("int", &receiver));
            };
            Ok(Value::Float(*n as f64))
        }

        // ── Float methods (19, 21, 23–25) ───────────────────
        19 => {
            // float.to_string
            let Value::Float(n) = &receiver else {
                return Err(type_error("float", &receiver));
            };
            Ok(Value::String(n.to_string()))
        }
        21 => {
            // float.abs
            let Value::Float(n) = &receiver else {
                return Err(type_error("float", &receiver));
            };
            Ok(Value::Float(n.abs()))
        }
        23 => {
            // float.floor
            let Value::Float(n) = &receiver else {
                return Err(type_error("float", &receiver));
            };
            Ok(Value::Int(n.floor() as i64))
        }
        24 => {
            // float.ceil
            let Value::Float(n) = &receiver else {
                return Err(type_error("float", &receiver));
            };
            Ok(Value::Int(n.ceil() as i64))
        }
        25 => {
            // float.round
            let Value::Float(n) = &receiver else {
                return Err(type_error("float", &receiver));
            };
            Ok(Value::Int(n.round() as i64))
        }

        _ => Err(RuntimeError::err(
            RuntimeErrorCode::R190,
            format!("unknown method id {method_id}"),
        )
        .build()),
    }
}

fn result_ok(value: Value) -> Value {
    Value::Enum {
        type_idx: 0xFFFF,
        variant_idx: 0,
        payload: Some(Box::new(value)),
    }
}

fn result_err(message: String) -> Value {
    Value::Enum {
        type_idx: 0xFFFF,
        variant_idx: 1,
        payload: Some(Box::new(Value::String(message))),
    }
}

fn type_error(expected: &str, got: &Value) -> RuntimeError {
    RuntimeError::err(
        RuntimeErrorCode::R120,
        format!("method expected {expected} receiver, got {}", type_name(got)),
    )
    .build()
}

fn arg_type_error(expected: &str, got: &Value) -> RuntimeError {
    RuntimeError::err(
        RuntimeErrorCode::R120,
        format!("method expected {expected} argument, got {}", type_name(got)),
    )
    .build()
}

fn type_name(val: &Value) -> &'static str {
    match val {
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Bool(_) => "bool",
        Value::String(_) => "string",
        Value::None => "None",
        Value::Unit => "()",
        Value::List(_) => "list",
        Value::Object(_) => "object",
        Value::Function(_) => "function",
        Value::Enum { .. } => "enum",
    }
}

fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Int(n) => n.to_string(),
        Value::Float(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::None => "None".to_string(),
        Value::Unit => "()".to_string(),
        other => format!("{other}"),
    }
}
