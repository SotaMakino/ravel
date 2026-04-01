use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Null,
    Undefined,
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Func {
        params: Vec<String>,
        body: crate::parser::ast::AstNode,
    },
    Builtin(fn(&[Value]) -> Result<Value, String>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Undefined => write!(f, "undefined"),
            Value::Object(entries) => {
                let parts: Vec<_> = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{ {} }}", parts.join(", "))
            }
            Value::Array(items) => {
                let parts: Vec<_> = items.iter().map(|i| format!("{}", i)).collect();
                write!(f, "[{}]", parts.join(", "))
            }
            Value::Func { params, .. } => {
                write!(f, "[function ({})]", params.join(", "))
            }
            Value::Builtin(_) => write!(f, "[builtin]"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_number_int() {
        assert_eq!(format!("{}", Value::Number(42.0)), "42");
    }

    #[test]
    fn test_display_number_float() {
        assert_eq!(format!("{}", Value::Number(3.14)), "3.14");
    }

    #[test]
    fn test_display_string() {
        assert_eq!(format!("{}", Value::Str("hello".into())), "hello");
    }

    #[test]
    fn test_display_bool() {
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Bool(false)), "false");
    }

    #[test]
    fn test_display_null() {
        assert_eq!(format!("{}", Value::Null), "null");
    }

    #[test]
    fn test_display_undefined() {
        assert_eq!(format!("{}", Value::Undefined), "undefined");
    }

    #[test]
    fn test_display_object() {
        let mut map = HashMap::new();
        map.insert("x".into(), Value::Number(1.0));
        map.insert("y".into(), Value::Number(2.0));
        let s = format!("{}", Value::Object(map));
        assert!(s.contains("x: 1"));
        assert!(s.contains("y: 2"));
    }

    #[test]
    fn test_display_array() {
        let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(format!("{}", arr), "[1, 2]");
    }

    #[test]
    fn test_display_builtin() {
        assert_eq!(
            format!("{}", Value::Builtin(|_| Ok(Value::Undefined))),
            "[builtin]"
        );
    }

    #[test]
    fn test_eq_numbers() {
        assert_eq!(Value::Number(1.0), Value::Number(1.0));
        assert_ne!(Value::Number(1.0), Value::Number(2.0));
    }

    #[test]
    fn test_eq_strings() {
        assert_eq!(Value::Str("hi".into()), Value::Str("hi".into()));
        assert_ne!(Value::Str("hi".into()), Value::Str("bye".into()));
    }

    #[test]
    fn test_eq_bools() {
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }

    #[test]
    fn test_eq_null() {
        assert_eq!(Value::Null, Value::Null);
    }

    #[test]
    fn test_eq_undefined() {
        assert_eq!(Value::Undefined, Value::Undefined);
    }

    #[test]
    fn test_eq_arrays() {
        let a = Value::Array(vec![Value::Number(1.0)]);
        let b = Value::Array(vec![Value::Number(1.0)]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_eq_objects() {
        let mut m1 = HashMap::new();
        m1.insert("a".into(), Value::Number(1.0));
        let mut m2 = HashMap::new();
        m2.insert("a".into(), Value::Number(1.0));
        assert_eq!(Value::Object(m1), Value::Object(m2));
    }

    #[test]
    fn test_ne_different_types() {
        assert_ne!(Value::Number(0.0), Value::Str("0".into()));
    }
}
