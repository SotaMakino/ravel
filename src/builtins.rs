use crate::value::Value;
use std::collections::HashMap;

pub fn console_log(args: &[Value]) -> Result<Value, String> {
    let parts: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    println!("{}", parts.join(" "));
    Ok(Value::Undefined)
}

pub fn create_console() -> Value {
    let mut obj = HashMap::new();
    obj.insert("log".to_string(), Value::Builtin(console_log));
    Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_console_has_log() {
        let console = create_console();
        match console {
            Value::Object(map) => {
                assert!(map.contains_key("log"));
                assert!(matches!(map.get("log"), Some(Value::Builtin(_))));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_console_log_returns_undefined() {
        let result = console_log(&[Value::Number(42.0)]);
        assert_eq!(result.unwrap(), Value::Undefined);
    }
}
