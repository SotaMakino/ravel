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
