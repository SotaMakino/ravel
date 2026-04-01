use crate::timer::{self, TimerState};
use crate::value::{BuiltinFn, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::time::{sleep, Duration};

#[derive(Clone, Copy)]
struct EnvPtr(*mut crate::env::Env);
unsafe impl Send for EnvPtr {}
unsafe impl Sync for EnvPtr {}

fn make_timer_builtins(state: TimerState, handle: Handle, parent_env: EnvPtr) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    let state_clone = state.clone();
    let handle_clone = handle.clone();
    let env_ptr = parent_env;
    let set_timeout: BuiltinFn = Arc::new(move |args| {
        builtin_set_timeout(args, state_clone.clone(), handle_clone.clone(), env_ptr)
    });
    map.insert("setTimeout".to_string(), Value::Builtin(set_timeout));

    let state_clone = state.clone();
    let handle_clone = handle.clone();
    let env_ptr = parent_env;
    let set_interval: BuiltinFn = Arc::new(move |args| {
        builtin_set_interval(args, state_clone.clone(), handle_clone.clone(), env_ptr)
    });
    map.insert("setInterval".to_string(), Value::Builtin(set_interval));

    let state_clone = state.clone();
    let clear_timeout: BuiltinFn = Arc::new(move |args| {
        builtin_clear(args, state_clone.clone())
    });
    map.insert("clearTimeout".to_string(), Value::Builtin(clear_timeout));

    let state_clone = state.clone();
    let clear_interval: BuiltinFn = Arc::new(move |args| {
        builtin_clear(args, state_clone.clone())
    });
    map.insert("clearInterval".to_string(), Value::Builtin(clear_interval));

    map
}

fn extract_callback_and_args(args: &[Value]) -> Result<(Value, Vec<Value>), String> {
    if args.is_empty() {
        return Err("setTimeout/setInterval requires at least a callback argument".into());
    }
    let callback = args[0].clone();
    match &callback {
        Value::Func { .. } | Value::Builtin(_) => {}
        _ => return Err("First argument must be a function".into()),
    }
    let call_args = args[1..].to_vec();
    Ok((callback, call_args))
}

fn spawn_callback(
    callback: Value,
    args: Vec<Value>,
    parent_env: EnvPtr,
) {
    match callback {
        Value::Func { params, body } => {
            use crate::env::Env;
            use crate::interpreter::Interpreter;
            let mut child_env = Env::with_parent(parent_env.0);
            for (param, arg) in params.iter().zip(args.iter()) {
                child_env.define(param, arg.clone());
            }
            let mut interp = Interpreter::new(&mut child_env);
            if let Err(e) = interp.execute(&body) {
                eprintln!("Timer error: {}", e);
            }
        }
        Value::Builtin(f) => {
            if let Err(e) = f(&args) {
                eprintln!("Timer error: {}", e);
            }
        }
        _ => {}
    }
}

fn builtin_set_timeout(args: &[Value], state: TimerState, handle: Handle, parent_env: EnvPtr) -> Result<Value, String> {
    let (callback, call_args) = extract_callback_and_args(args)?;
    let delay_ms = if args.len() >= 2 {
        match &args[1] {
            Value::Number(n) => *n as u64,
            _ => 0,
        }
    } else {
        0
    };
    let id = timer::next_timer_id();
    let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();
    let state_clone = state.clone();

    let task = handle.spawn(async move {
        sleep(Duration::from_millis(delay_ms)).await;
        if cancelled_clone.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        state_clone.entries.lock().unwrap().remove(&id);
        spawn_callback(callback, call_args, parent_env);
    });

    state.register(id, cancelled, task.abort_handle());
    Ok(Value::Number(id as f64))
}

fn builtin_set_interval(args: &[Value], state: TimerState, handle: Handle, parent_env: EnvPtr) -> Result<Value, String> {
    let (callback, call_args) = extract_callback_and_args(args)?;
    let interval_ms = if args.len() >= 2 {
        match &args[1] {
            Value::Number(n) => *n as u64,
            _ => 0,
        }
    } else {
        0
    };
    let id = timer::next_timer_id();
    let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    let task = handle.spawn(async move {
        loop {
            sleep(Duration::from_millis(interval_ms)).await;
            if cancelled_clone.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            spawn_callback(callback.clone(), call_args.clone(), parent_env);
        }
    });

    state.register(id, cancelled, task.abort_handle());
    Ok(Value::Number(id as f64))
}

fn builtin_clear(args: &[Value], state: TimerState) -> Result<Value, String> {
    if !args.is_empty() {
        if let Value::Number(n) = &args[0] {
            state.cancel(*n as u32);
        }
    }
    Ok(Value::Undefined)
}

pub fn console_log(args: &[Value]) -> Result<Value, String> {
    let parts: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    println!("{}", parts.join(" "));
    Ok(Value::Undefined)
}

pub fn create_console() -> Value {
    let mut obj = HashMap::new();
    let log_fn: BuiltinFn = Arc::new(|args| console_log(args));
    obj.insert("log".to_string(), Value::Builtin(log_fn));
    Value::Object(obj)
}

pub fn create_timer_globals(state: TimerState, handle: Handle, parent_env: *mut crate::env::Env) -> HashMap<String, Value> {
    make_timer_builtins(state, handle, EnvPtr(parent_env))
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
