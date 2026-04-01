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
            _ => false,
        }
    }
}
