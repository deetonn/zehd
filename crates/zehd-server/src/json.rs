use serde_json::Value as JsonValue;
use zehd_rune::value::Value;

/// Convert a zehd `Value` to a `serde_json::Value`.
///
/// Returns `None` for `Value::Unit` — the caller should respond with 204 No Content.
pub fn value_to_json(value: &Value) -> Option<JsonValue> {
    match value {
        Value::Unit => None,
        Value::Int(n) => Some(JsonValue::Number((*n).into())),
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                Some(JsonValue::Null)
            } else {
                serde_json::Number::from_f64(*f).map(JsonValue::Number)
            }
        }
        Value::Bool(b) => Some(JsonValue::Bool(*b)),
        Value::String(s) => Some(JsonValue::String(s.clone())),
        Value::None => Some(JsonValue::Null),
        Value::List(items) => {
            let arr: Vec<JsonValue> = items
                .iter()
                .map(|v| value_to_json(v).unwrap_or(JsonValue::Null))
                .collect();
            Some(JsonValue::Array(arr))
        }
        Value::Object(fields) => {
            let mut map = serde_json::Map::new();
            for (key, val) in fields {
                map.insert(
                    key.clone(),
                    value_to_json(val).unwrap_or(JsonValue::Null),
                );
            }
            Some(JsonValue::Object(map))
        }
        Value::Function(_) => Some(JsonValue::Null),
        Value::Enum { type_idx, variant_idx, payload } => {
            match (*type_idx, *variant_idx) {
                // Option::None → JSON null
                (0xFFFE, 1) => Some(JsonValue::Null),
                // Option::Some / Result::Ok → unwrap payload
                (0xFFFE, 0) | (0xFFFF, 0) => {
                    match payload {
                        Some(inner) => value_to_json(inner).or(Some(JsonValue::Null)),
                        None => Some(JsonValue::Null),
                    }
                }
                // Result::Err → serialize as { "error": payload }
                (0xFFFF, 1) => {
                    let mut map = serde_json::Map::new();
                    let inner = payload.as_ref()
                        .map(|v| value_to_json(v).unwrap_or(JsonValue::Null))
                        .unwrap_or(JsonValue::Null);
                    map.insert("error".to_string(), inner);
                    Some(JsonValue::Object(map))
                }
                // User-defined enums → serialize as { "variant": variant_idx, "value": payload }
                _ => {
                    let mut map = serde_json::Map::new();
                    map.insert("variant".to_string(), JsonValue::from(*variant_idx));
                    if let Some(inner) = payload {
                        map.insert("value".to_string(), value_to_json(inner).unwrap_or(JsonValue::Null));
                    }
                    Some(JsonValue::Object(map))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_to_json() {
        assert_eq!(value_to_json(&Value::Int(42)), Some(JsonValue::from(42)));
    }

    #[test]
    fn negative_int_to_json() {
        assert_eq!(value_to_json(&Value::Int(-1)), Some(JsonValue::from(-1)));
    }

    #[test]
    fn float_to_json() {
        assert_eq!(
            value_to_json(&Value::Float(3.14)),
            Some(JsonValue::from(3.14))
        );
    }

    #[test]
    fn float_nan_to_null() {
        assert_eq!(
            value_to_json(&Value::Float(f64::NAN)),
            Some(JsonValue::Null)
        );
    }

    #[test]
    fn float_inf_to_null() {
        assert_eq!(
            value_to_json(&Value::Float(f64::INFINITY)),
            Some(JsonValue::Null)
        );
    }

    #[test]
    fn bool_true_to_json() {
        assert_eq!(
            value_to_json(&Value::Bool(true)),
            Some(JsonValue::Bool(true))
        );
    }

    #[test]
    fn bool_false_to_json() {
        assert_eq!(
            value_to_json(&Value::Bool(false)),
            Some(JsonValue::Bool(false))
        );
    }

    #[test]
    fn string_to_json() {
        assert_eq!(
            value_to_json(&Value::String("hello".into())),
            Some(JsonValue::String("hello".into()))
        );
    }

    #[test]
    fn none_to_null() {
        assert_eq!(value_to_json(&Value::None), Some(JsonValue::Null));
    }

    #[test]
    fn unit_to_none() {
        assert_eq!(value_to_json(&Value::Unit), None);
    }

    #[test]
    fn list_to_array() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(
            value_to_json(&list),
            Some(JsonValue::Array(vec![
                JsonValue::from(1),
                JsonValue::from(2),
                JsonValue::from(3),
            ]))
        );
    }

    #[test]
    fn object_to_json_object() {
        let obj = Value::Object(vec![
            ("name".into(), Value::String("zehd".into())),
            ("version".into(), Value::Int(1)),
        ]);
        let json = value_to_json(&obj).unwrap();
        assert_eq!(json["name"], JsonValue::String("zehd".into()));
        assert_eq!(json["version"], JsonValue::from(1));
    }

    #[test]
    fn nested_list_in_object() {
        let obj = Value::Object(vec![(
            "items".into(),
            Value::List(vec![Value::Bool(true), Value::None]),
        )]);
        let json = value_to_json(&obj).unwrap();
        assert_eq!(
            json["items"],
            JsonValue::Array(vec![JsonValue::Bool(true), JsonValue::Null])
        );
    }

    #[test]
    fn function_to_null() {
        assert_eq!(
            value_to_json(&Value::Function(0)),
            Some(JsonValue::Null)
        );
    }

    #[test]
    fn unit_in_list_becomes_null() {
        let list = Value::List(vec![Value::Unit, Value::Int(1)]);
        assert_eq!(
            value_to_json(&list),
            Some(JsonValue::Array(vec![JsonValue::Null, JsonValue::from(1)]))
        );
    }
}
