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
