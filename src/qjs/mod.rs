use std::path::Path;

use rquickjs::{Ctx, Object, Result, Value, function::Rest};

pub mod timers;
pub mod fs;

pub use timers::{TimerMessage, TimerState, get_timer_state, set_timer_state};
pub use timers::setup_timers;
pub use fs::setup_fs;

pub fn value_to_string(v: &Value<'_>) -> String {
    if let Some(s) = v.as_string() {
        s.to_string().unwrap_or_else(|_| "[object]".to_string())
    } else if let Some(n) = v.as_int() {
        n.to_string()
    } else if let Some(n) = v.as_float() {
        n.to_string()
    } else if let Some(b) = v.as_bool() {
        b.to_string()
    } else if v.is_null() {
        "null".to_string()
    } else if v.is_undefined() {
        "undefined".to_string()
    } else {
        format!("[{:?}]", v.type_of())
    }
}

pub fn setup_console<'js>(ctx: &Ctx<'js>) -> Result<()> {
    let console = Object::new(ctx.clone())?;
    console.set(
        "log",
        rquickjs::function::Func::new(|args: Rest<Value<'_>>| -> rquickjs::Result<()> {
            let parts: Vec<String> = args.0.iter().map(|v| value_to_string(v)).collect();
            println!("{}", parts.join(" "));
            Ok(())
        }),
    )?;
    ctx.globals().set("console", console)?;
    Ok(())
}

pub fn setup_full_environment<'js>(ctx: &Ctx<'js>, root: &Path) -> Result<()> {
    setup_console(ctx)?;
    setup_timers(ctx)?;
    setup_fs(ctx, root)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::Context;

    #[test]
    fn test_value_to_string_string() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("\"hello\"").unwrap();
            assert_eq!(value_to_string(&v), "hello");
        });
    }

    #[test]
    fn test_value_to_string_int() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("42").unwrap();
            assert_eq!(value_to_string(&v), "42");
        });
    }

    #[test]
    fn test_value_to_string_float() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("3.14").unwrap();
            assert_eq!(value_to_string(&v), "3.14");
        });
    }

    #[test]
    fn test_value_to_string_bool() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("true").unwrap();
            assert_eq!(value_to_string(&v), "true");
        });
    }

    #[test]
    fn test_value_to_string_null() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("null").unwrap();
            assert_eq!(value_to_string(&v), "null");
        });
    }

    #[test]
    fn test_value_to_string_undefined() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("undefined").unwrap();
            assert_eq!(value_to_string(&v), "undefined");
        });
    }

    #[test]
    fn test_value_to_string_object() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("({})").unwrap();
            assert!(value_to_string(&v).contains("Object"));
        });
    }

    #[test]
    fn test_setup_console() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_console(&ctx).unwrap();
            let console: Object = ctx.globals().get("console").unwrap();
            assert!(console.get::<_, rquickjs::Function>("log").is_ok());
        });
    }

    #[test]
    fn test_setup_full_environment() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        let root = std::env::current_dir().unwrap();
        ctx.with(|ctx| {
            setup_full_environment(&ctx, &root).unwrap();
            assert!(ctx.globals().get::<_, Object>("console").is_ok());
            assert!(ctx.globals().get::<_, Object>("fs").is_ok());
        });
    }

    #[test]
    fn test_value_to_string_negative_int() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("-42").unwrap();
            assert_eq!(value_to_string(&v), "-42");
        });
    }

    #[test]
    fn test_value_to_string_zero() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("0").unwrap();
            assert_eq!(value_to_string(&v), "0");
        });
    }

    #[test]
    fn test_value_to_string_false() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("false").unwrap();
            assert_eq!(value_to_string(&v), "false");
        });
    }

    #[test]
    fn test_value_to_string_empty_string() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("\"\"").unwrap();
            assert_eq!(value_to_string(&v), "");
        });
    }

    #[test]
    fn test_value_to_string_array() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("[1, 2, 3]").unwrap();
            assert!(value_to_string(&v).contains("Array"));
        });
    }

    #[test]
    fn test_value_to_string_empty_array() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("[]").unwrap();
            assert!(value_to_string(&v).contains("Array"));
        });
    }

    #[test]
    fn test_value_to_string_function() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("(() => {})").unwrap();
            assert!(value_to_string(&v).contains("Function"));
        });
    }

    #[test]
    fn test_value_to_string_nan() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("NaN").unwrap();
            let result = value_to_string(&v);
            assert!(result.contains("NaN") || result == "NaN");
        });
    }

    #[test]
    fn test_value_to_string_infinity() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("Infinity").unwrap();
            let result = value_to_string(&v);
            assert!(result == "inf" || result.contains("inf"));
        });
    }

    #[test]
    fn test_value_to_string_object_with_properties() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("({ foo: 'bar' })").unwrap();
            assert!(value_to_string(&v).contains("Object"));
        });
    }

    #[test]
    fn test_value_to_string_nested_object() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("({ a: { b: 1 } })").unwrap();
            assert!(value_to_string(&v).contains("Object"));
        });
    }

    #[test]
    fn test_value_to_string_large_float() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("3.14159265358979").unwrap();
            assert_eq!(value_to_string(&v), "3.14159265358979");
        });
    }

    #[test]
    fn test_value_to_string_negative_float() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let v: Value = ctx.eval("-3.14").unwrap();
            assert_eq!(value_to_string(&v), "-3.14");
        });
    }
}
