use rquickjs::{Ctx, Error, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use crate::console::value_to_string;

/// Hidden property stamped on a rejected promise so a later "handled"
/// callback can find and drop the matching pending report.
const REJECTION_ID_PROP: &str = "__ravel_rejection_id";

static PENDING_REJECTIONS: LazyLock<Mutex<HashMap<u64, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static NEXT_REJECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Render a thrown JS value as `Name: message` followed by its stack.
/// Non-Error throws (`throw "boom"`) fall back to their string form.
pub fn format_thrown(value: &Value<'_>) -> String {
    let Some(exception) = value.as_exception() else {
        return value_to_string(value);
    };

    let name = exception
        .as_object()
        .get::<_, String>("name")
        .unwrap_or_else(|_| "Error".to_string());
    let message = exception.message().unwrap_or_default();

    let mut out = if message.is_empty() {
        name
    } else {
        format!("{}: {}", name, message)
    };

    if let Some(stack) = exception.stack() {
        let stack = strip_internal_frames(&stack);
        if !stack.is_empty() {
            out.push('\n');
            out.push_str(&stack);
        }
    }

    out
}

/// Cut the stack at ravel's own plumbing (the `__ravel_*` timer bootstrap).
/// Everything below such a frame is runtime internals, not user code.
fn strip_internal_frames(stack: &str) -> String {
    stack
        .lines()
        .take_while(|line| !line.contains("__ravel_"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// Turn an `rquickjs::Error` into a printable report, pulling the live
/// exception out of the context when the error carries one.
pub fn format_error(ctx: &Ctx<'_>, err: &Error) -> String {
    if err.is_exception() {
        format_thrown(&ctx.catch())
    } else {
        err.to_string()
    }
}

/// Print an uncaught error to stderr.
pub fn report_uncaught(ctx: &Ctx<'_>, err: &Error) {
    eprintln!("Uncaught {}", format_error(ctx, err));
}

/// Record a rejection that has no handler yet. QuickJS calls this at
/// rejection time, so the report is only real if nothing handles the
/// promise later — see [`forget_rejection`].
pub fn track_rejection(promise: &Value<'_>, reason: &Value<'_>) {
    let Some(obj) = promise.as_object() else {
        return;
    };
    let id = NEXT_REJECTION_ID.fetch_add(1, Ordering::SeqCst);
    if obj.set(REJECTION_ID_PROP, id).is_err() {
        return;
    }
    PENDING_REJECTIONS
        .lock()
        .unwrap()
        .insert(id, format_thrown(reason));
}

/// Drop a pending report because the promise got a handler after all.
pub fn forget_rejection(promise: &Value<'_>) {
    let Some(obj) = promise.as_object() else {
        return;
    };
    if let Ok(id) = obj.get::<_, u64>(REJECTION_ID_PROP) {
        PENDING_REJECTIONS.lock().unwrap().remove(&id);
    }
}

/// Print every rejection that was never handled. Returns true if any were
/// reported, which callers treat as a failed run.
pub fn report_pending_rejections() -> bool {
    let mut pending = PENDING_REJECTIONS.lock().unwrap();
    if pending.is_empty() {
        return false;
    }
    let mut ids: Vec<u64> = pending.keys().copied().collect();
    ids.sort_unstable();
    for id in &ids {
        eprintln!("Unhandled promise rejection: {}", pending[id]);
    }
    pending.clear();
    true
}

/// Forget every pending rejection without reporting. Used by the REPL so one
/// line's rejection is not blamed on the next.
pub fn clear_pending_rejections() {
    PENDING_REJECTIONS.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{Context, Runtime};

    fn with_ctx<F, R>(f: F) -> R
    where
        F: FnOnce(Ctx<'_>) -> R,
    {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(f)
    }

    /// The pending-rejection map is process-global, so tests that touch it
    /// must not run concurrently.
    static REJECTION_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_rejection_registry<F: FnOnce()>(f: F) {
        let _guard = REJECTION_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        clear_pending_rejections();
        f();
        clear_pending_rejections();
    }

    #[test]
    fn test_format_thrown_includes_name_and_message() {
        with_ctx(|ctx| {
            let err: Error = ctx
                .eval::<Value, _>("null.x")
                .expect_err("expected a TypeError");
            let report = format_error(&ctx, &err);
            assert!(report.starts_with("TypeError: "), "got: {}", report);
        });
    }

    #[test]
    fn test_format_thrown_includes_stack_frames() {
        with_ctx(|ctx| {
            let err: Error = ctx
                .eval::<Value, _>("function a(){ null.x } function b(){ a() } b()")
                .expect_err("expected a TypeError");
            let report = format_error(&ctx, &err);
            assert!(report.contains("\n    at "), "got: {}", report);
        });
    }

    #[test]
    fn test_format_thrown_custom_error_name() {
        with_ctx(|ctx| {
            let err: Error = ctx
                .eval::<Value, _>("throw new RangeError('too big')")
                .expect_err("expected a RangeError");
            let report = format_error(&ctx, &err);
            assert!(report.starts_with("RangeError: too big"), "got: {}", report);
        });
    }

    #[test]
    fn test_format_thrown_non_error_value() {
        with_ctx(|ctx| {
            let err: Error = ctx
                .eval::<Value, _>("throw 'boom'")
                .expect_err("expected a thrown string");
            let report = format_error(&ctx, &err);
            assert_eq!(report, "boom");
        });
    }

    #[test]
    fn test_strip_internal_frames_cuts_at_ravel_plumbing() {
        let stack = "    at <anonymous> (t.js:1:19)\n    at apply (native)\n    at __ravel_fire_timer (eval_script:9:30)\n    at <eval> (eval_script:1:1)";
        let stripped = strip_internal_frames(stack);
        assert_eq!(
            stripped,
            "    at <anonymous> (t.js:1:19)\n    at apply (native)"
        );
    }

    #[test]
    fn test_strip_internal_frames_keeps_user_only_stack() {
        let stack = "    at a (err.js:1:14)\n    at b (err.js:2:15)";
        assert_eq!(strip_internal_frames(stack), stack);
    }

    #[test]
    fn test_format_error_non_exception() {
        with_ctx(|ctx| {
            let report = format_error(&ctx, &Error::Unknown);
            assert!(!report.is_empty());
        });
    }

    #[test]
    fn test_track_and_report_rejection() {
        with_rejection_registry(|| {
            with_ctx(|ctx| {
                let promise: Value = ctx.eval("Promise.resolve()").unwrap();
                let reason: Value = ctx.eval("new Error('nope')").unwrap();
                track_rejection(&promise, &reason);
            });
            assert!(report_pending_rejections());
            // Draining leaves nothing behind.
            assert!(!report_pending_rejections());
        });
    }

    #[test]
    fn test_forget_rejection_removes_pending() {
        with_rejection_registry(|| {
            with_ctx(|ctx| {
                let promise: Value = ctx.eval("Promise.resolve()").unwrap();
                let reason: Value = ctx.eval("new Error('handled later')").unwrap();
                track_rejection(&promise, &reason);
                forget_rejection(&promise);
            });
            assert!(!report_pending_rejections());
        });
    }

    #[test]
    fn test_clear_pending_rejections() {
        with_rejection_registry(|| {
            with_ctx(|ctx| {
                let promise: Value = ctx.eval("Promise.resolve()").unwrap();
                let reason: Value = ctx.eval("new Error('dropped')").unwrap();
                track_rejection(&promise, &reason);
            });
            clear_pending_rejections();
            assert!(!report_pending_rejections());
        });
    }
}
