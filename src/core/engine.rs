use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use rquickjs::{AsyncContext, AsyncRuntime, Ctx, Object, Result};
use tokio::sync::mpsc;

use crate::console::setup_console;
use crate::encoding::setup_encoding;
use crate::error::{format_thrown, forget_rejection, report_uncaught, track_rejection};
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
        runtime
            .set_host_promise_rejection_tracker(Some(Box::new(
                |_ctx, promise, reason, is_handled| {
                    if is_handled {
                        forget_rejection(&promise);
                    } else {
                        track_rejection(&promise, &reason);
                    }
                },
            )))
            .await;
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
        setup_encoding(ctx)?;
        setup_timers(ctx)?;
        setup_fs(ctx, root)?;
        setup_jsx_runtime(ctx)?;
        Ok(())
    }

    /// `[execPath, scriptPath, ...userArgs]`, matching Node's shape. Args that
    /// belong to ravel itself (flags, the script path) are not passed through.
    /// In the REPL there is no script, so only the exec path is present.
    fn build_argv(file: &str) -> Vec<String> {
        let raw: Vec<String> = std::env::args().collect();
        let mut argv: Vec<String> = raw.first().cloned().into_iter().collect();

        if file.is_empty() {
            return argv;
        }
        argv.push(file.to_string());

        // Locate the script in the real args so anything after it is a user arg.
        let script_idx = raw.iter().position(|arg| {
            Path::new(arg)
                .canonicalize()
                .is_ok_and(|p| p.to_string_lossy() == file)
        });
        if let Some(idx) = script_idx {
            argv.extend(raw.iter().skip(idx + 1).cloned());
        }
        argv
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

        // Built through the object API rather than eval'd JSON so values
        // containing quotes or backslashes cannot break out.
        let process = Object::new(ctx.clone())?;

        let env = Object::new(ctx.clone())?;
        for (k, v) in env_vars {
            env.set(k, v)?;
        }
        process.set("env", env)?;
        process.set("argv", Self::build_argv(file))?;
        process.set(
            "exit",
            // Opt, not Option: Option requires the argument to be present.
            rquickjs::function::Func::new(|code: rquickjs::function::Opt<i32>| -> () {
                // println! is line-buffered, but flush anyway since
                // process::exit skips destructors.
                let _ = std::io::stdout().flush();
                std::process::exit(code.0.unwrap_or(0));
            }),
        )?;
        ctx.globals().set("process", process)?;

        let _: Result<()> = ctx.eval(format!(
            "var ravel = {{ version: {:?}, build: {} }};",
            RAVEL_VERSION, build_mode
        ));

        Ok(())
    }

    /// Run queued microtasks (promise callbacks, `await` continuations) and
    /// wait for spawned futures (async fs) to finish. Returns false if a job
    /// threw an uncaught error.
    ///
    /// Two phases, because `execute_pending_job` cannot tell "queue empty"
    /// from "futures still in flight" -- it returns `Ok(false)` for both. The
    /// first phase drains ready jobs so their errors can be reported here
    /// rather than by `idle`, which uses `println!` and would corrupt build
    /// output on stdout. The second parks until the scheduler is empty, so a
    /// pending read cannot be abandoned at exit.
    pub async fn run_pending_jobs(&self) -> bool {
        let mut ok = true;
        loop {
            match self.runtime.execute_pending_job().await {
                Ok(true) => continue,
                Ok(false) => break,
                Err(exception) => {
                    ok = false;
                    let ctx = exception.0.clone();
                    rquickjs::async_with!(ctx => |ctx| {
                        eprintln!("Uncaught {}", format_thrown(&ctx.catch()));
                    })
                    .await;
                }
            }
        }
        self.runtime.idle().await;
        ok
    }

    /// Returns false if any timer callback threw an uncaught error.
    pub async fn drain_timers(&mut self) -> bool {
        let mut ok = true;
        loop {
            let fired = tokio::select! {
                Some(msg) = self.timer_rx.recv() => Some(msg),
                _ = tokio::time::sleep(Duration::from_millis(10)) => None,
            };

            let Some(msg) = fired else {
                let has_pending = get_timer_state().is_some_and(|state| state.has_pending());
                if !has_pending {
                    break;
                }
                continue;
            };

            let ctx = self.context.clone();
            ok &= rquickjs::async_with!(ctx => |ctx| {
                let mut fired_ok = true;
                match msg {
                    TimerMessage::FireTimeout(id) => {
                        if let Err(e) = ctx.eval::<(), _>(format!("__ravel_fire_timer({})", id)) {
                            report_uncaught(&ctx, &e);
                            fired_ok = false;
                        }
                        if let Some(state) = get_timer_state() {
                            state.entries.lock().unwrap().remove(&id);
                        }
                    }
                    TimerMessage::FireInterval(id) => {
                        if let Err(e) = ctx.eval::<(), _>(format!("__ravel_fire_interval({})", id)) {
                            report_uncaught(&ctx, &e);
                            fired_ok = false;
                        }
                    }
                }
                fired_ok
            })
            .await;

            // A timer callback can queue promise jobs; run them before the
            // next tick so `await` inside a callback makes progress.
            ok &= self.run_pending_jobs().await;
        }
        ok
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