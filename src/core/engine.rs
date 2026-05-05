use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use rquickjs::{AsyncContext, AsyncRuntime, Ctx, Result};
use tokio::sync::mpsc;

use crate::console::setup_console;
use crate::fs::setup_fs;
use crate::jsx::setup_jsx_runtime;
use crate::timer::{TimerMessage, TimerState, get_timer_state, set_timer_state, setup_timers};

pub const RAVEL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Engine {
    pub runtime: AsyncRuntime,
    pub context: AsyncContext,
    timer_rx: mpsc::UnboundedReceiver<TimerMessage>,
}

impl Engine {
    pub async fn new() -> Self {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let context = AsyncContext::full(&runtime)
            .await
            .expect("Failed to create context");
        let (timer_state, timer_rx) = TimerState::new();
        set_timer_state(timer_state);
        Self {
            runtime,
            context,
            timer_rx,
        }
    }

    pub fn setup_all_apis<'js>(ctx: &Ctx<'js>, root: &Path) -> Result<()> {
        setup_console(ctx)?;
        setup_timers(ctx)?;
        setup_fs(ctx, root)?;
        setup_jsx_runtime(ctx)?;
        Ok(())
    }

    pub fn inject_globals<'js>(
        ctx: &Ctx<'js>,
        file: &str,
        dir: &str,
        build_mode: bool,
    ) -> Result<()> {
        let _: Result<()> = ctx.eval(format!("var __filename = {:?};", file));
        let _: Result<()> = ctx.eval(format!("var __dirname = {:?};", dir));

        let mut env_vars: HashMap<String, String> = std::env::vars().collect();
        if build_mode {
            env_vars.insert("RAVEL_BUILD".to_string(), "1".to_string());
        }

        let mut env_parts = Vec::new();
        for (k, v) in &env_vars {
            let escaped_k = k.replace('\\', "\\\\").replace('"', "\\\"");
            let escaped_v = v.replace('\\', "\\\\").replace('"', "\\\"");
            env_parts.push(format!("\"{}\":\"{}\"", escaped_k, escaped_v));
        }
        let env_json = format!("{{{}}}", env_parts.join(","));
        let _: Result<()> = ctx.eval(format!("var process = {{ env: {} }};", env_json));

        let _: Result<()> = ctx.eval(format!(
            "var ravel = {{ version: {:?}, build: {} }};",
            RAVEL_VERSION, build_mode
        ));

        Ok(())
    }

    pub async fn drain_timers(&mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.timer_rx.recv() => {
                    let ctx = self.context.clone();
                    rquickjs::async_with!(ctx => |ctx| {
                        match msg {
                            TimerMessage::FireTimeout(id) => {
                                let _: Result<()> = ctx.eval(format!("__ravel_fire_timer({})", id));
                                if let Some(state) = get_timer_state() {
                                    state.entries.lock().unwrap().remove(&id);
                                }
                            }
                            TimerMessage::FireInterval(id) => {
                                let _: Result<()> = ctx.eval(format!("__ravel_fire_interval({})", id));
                            }
                        }
                    })
                    .await;
                }
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    if let Some(state) = get_timer_state() {
                        if !state.has_pending() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::Context;

    fn with_ctx<F, R>(f: F) -> R
    where
        F: FnOnce(Ctx) -> R,
    {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(f)
    }

    #[test]
    fn test_inject_globals_sets_filename() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "/path/to/file.js", "/path/to", false).unwrap();
            let filename: String = ctx.eval("__filename").unwrap();
            assert_eq!(filename, "/path/to/file.js");
        })
    }

    #[test]
    fn test_inject_globals_sets_dirname() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "/path/to/file.js", "/path/to", false).unwrap();
            let dirname: String = ctx.eval("__dirname").unwrap();
            assert_eq!(dirname, "/path/to");
        })
    }

    #[test]
    fn test_inject_globals_process_env_is_object() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", false).unwrap();
            let typeof_env: String = ctx.eval("typeof process.env").unwrap();
            assert_eq!(typeof_env, "object");
        })
    }

    #[test]
    fn test_inject_globals_ravel_version() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", false).unwrap();
            let version: String = ctx.eval("ravel.version").unwrap();
            assert_eq!(version, RAVEL_VERSION);
        })
    }

    #[test]
    fn test_inject_globals_build_mode_false() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", false).unwrap();
            let build: bool = ctx.eval("ravel.build").unwrap();
            assert!(!build);
        })
    }

    #[test]
    fn test_inject_globals_build_mode_true() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", true).unwrap();
            let build: bool = ctx.eval("ravel.build").unwrap();
            assert!(build);
        })
    }

    #[test]
    fn test_inject_globals_ravel_build_env_in_build_mode() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", true).unwrap();
            let val: String = ctx.eval("process.env.RAVEL_BUILD").unwrap();
            assert_eq!(val, "1");
        })
    }

    #[test]
    fn test_inject_globals_no_ravel_build_env_in_normal_mode() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", false).unwrap();
            let result: rquickjs::Value = ctx.eval("process.env.RAVEL_BUILD").unwrap();
            assert!(
                result.is_undefined(),
                "RAVEL_BUILD should be undefined in normal mode"
            );
        })
    }

    #[test]
    fn test_setup_all_apis_injects_console() {
        with_ctx(|ctx| {
            let root = std::env::temp_dir();
            Engine::setup_all_apis(&ctx, &root).unwrap();
            let console: rquickjs::Object = ctx.globals().get("console").unwrap();
            assert!(console.get::<_, rquickjs::Function>("log").is_ok());
        })
    }

    #[test]
    fn test_setup_all_apis_injects_timers() {
        with_ctx(|ctx| {
            let root = std::env::temp_dir();
            Engine::setup_all_apis(&ctx, &root).unwrap();
            let set_timeout: rquickjs::Function = ctx.globals().get("setTimeout").unwrap();
            assert!(set_timeout.is_function());
        })
    }

    #[test]
    fn test_setup_all_apis_injects_fs() {
        with_ctx(|ctx| {
            let root = std::env::temp_dir();
            Engine::setup_all_apis(&ctx, &root).unwrap();
            let fs_obj: rquickjs::Object = ctx.globals().get("fs").unwrap();
            assert!(fs_obj.get::<_, rquickjs::Function>("readFile").is_ok());
        })
    }

    #[test]
    fn test_setup_all_apis_injects_note() {
        with_ctx(|ctx| {
            let root = std::env::temp_dir();
            Engine::setup_all_apis(&ctx, &root).unwrap();
            let note: rquickjs::Function = ctx.globals().get("note").unwrap();
            assert!(note.is_function());
        })
    }

    #[test]
    fn test_inject_globals_escapes_special_chars_in_paths() {
        with_ctx(|ctx| {
            Engine::inject_globals(
                &ctx,
                "/path/with \"quote/file.js",
                "/path/with \"quote",
                false,
            )
            .unwrap();
            let filename: String = ctx.eval("__filename").unwrap();
            assert_eq!(filename, "/path/with \"quote/file.js");
        })
    }
}