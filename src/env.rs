use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Env {
    values: HashMap<String, Value>,
    parent: Option<*mut Env>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: *mut Env) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.values.get(name) {
            return Some(v.clone());
        }
        if let Some(parent) = self.parent {
            unsafe { return (*parent).get(name) }
        }
        None
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(parent) = self.parent {
            unsafe { return (*parent).set(name, value) }
        }
        Err(format!("Undefined variable: {}", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_get() {
        let mut env = Env::new();
        env.define("x", Value::Number(42.0));
        assert_eq!(env.get("x"), Some(Value::Number(42.0)));
    }

    #[test]
    fn test_get_undefined() {
        let env = Env::new();
        assert_eq!(env.get("missing"), None);
    }

    #[test]
    fn test_set_existing() {
        let mut env = Env::new();
        env.define("x", Value::Number(1.0));
        env.set("x", Value::Number(2.0)).unwrap();
        assert_eq!(env.get("x"), Some(Value::Number(2.0)));
    }

    #[test]
    fn test_set_undefined() {
        let mut env = Env::new();
        assert!(env.set("x", Value::Number(1.0)).is_err());
    }

    #[test]
    fn test_parent_scope() {
        let mut parent = Env::new();
        parent.define("x", Value::Number(10.0));

        let mut child = Env::with_parent(&mut parent as *mut Env);
        child.define("y", Value::Number(20.0));

        assert_eq!(child.get("x"), Some(Value::Number(10.0)));
        assert_eq!(child.get("y"), Some(Value::Number(20.0)));
        assert_eq!(child.get("z"), None);
    }

    #[test]
    fn test_set_in_parent_scope() {
        let mut parent = Env::new();
        parent.define("x", Value::Number(1.0));

        let mut child = Env::with_parent(&mut parent as *mut Env);
        child.set("x", Value::Number(99.0)).unwrap();

        assert_eq!(parent.get("x"), Some(Value::Number(99.0)));
    }

    #[test]
    fn test_shadowing() {
        let mut parent = Env::new();
        parent.define("x", Value::Number(1.0));

        let mut child = Env::with_parent(&mut parent as *mut Env);
        child.define("x", Value::Number(2.0));

        assert_eq!(child.get("x"), Some(Value::Number(2.0)));
    }
}
