use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use rquickjs::{AsyncContext, AsyncRuntime, Ctx, Object, Result};

use crate::console::setup_console;
use crate::core::EventLoop;
use crate::encoding::setup_encoding;
use crate::error::{forget_rejection, track_rejection};
use crate::fs::setup_fs;
use crate::jsx::setup_jsx_runtime;
use crate::timer::{TimerState, set_timer_state, setup_timers};

pub const RAVEL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Engine {
    pub runtime: AsyncRuntime,
    pub context: AsyncContext,
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
        set_timer_state(TimerState::new());
        Self { runtime, context }
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
        base: &str,
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

        // `base` comes from ravel.json, so a build script and the server that
        // later serves it agree on one value instead of each keeping its own.
        let _: Result<()> = ctx.eval(format!(
            "var ravel = {{ version: {:?}, build: {}, base: {:?} }};",
            RAVEL_VERSION, build_mode, base
        ));

        Ok(())
    }

    /// Run the event loop until the script has nothing left to wait for.
    /// Returns false if any job or timer callback threw an uncaught error.
    pub async fn run_event_loop(&self) -> bool {
        EventLoop::new(&self.runtime, &self.context).run().await
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
            Engine::inject_globals(&ctx, "/path/to/file.js", "/path/to", false, "/").unwrap();
            let filename: String = ctx.eval("__filename").unwrap();
            assert_eq!(filename, "/path/to/file.js");
        })
    }

    #[test]
    fn test_inject_globals_sets_dirname() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "/path/to/file.js", "/path/to", false, "/").unwrap();
            let dirname: String = ctx.eval("__dirname").unwrap();
            assert_eq!(dirname, "/path/to");
        })
    }

    #[test]
    fn test_inject_globals_process_env_is_object() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", false, "/").unwrap();
            let typeof_env: String = ctx.eval("typeof process.env").unwrap();
            assert_eq!(typeof_env, "object");
        })
    }

    #[test]
    fn test_inject_globals_ravel_version() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", false, "/").unwrap();
            let version: String = ctx.eval("ravel.version").unwrap();
            assert_eq!(version, RAVEL_VERSION);
        })
    }

    #[test]
    fn test_inject_globals_exposes_base() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", false, "/my-repo/").unwrap();
            let base: String = ctx.eval("ravel.base").unwrap();
            assert_eq!(base, "/my-repo/");
        })
    }

    #[test]
    fn test_inject_globals_build_mode_false() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", false, "/").unwrap();
            let build: bool = ctx.eval("ravel.build").unwrap();
            assert!(!build);
        })
    }

    #[test]
    fn test_inject_globals_build_mode_true() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", true, "/").unwrap();
            let build: bool = ctx.eval("ravel.build").unwrap();
            assert!(build);
        })
    }

    #[test]
    fn test_inject_globals_ravel_build_env_in_build_mode() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", true, "/").unwrap();
            let val: String = ctx.eval("process.env.RAVEL_BUILD").unwrap();
            assert_eq!(val, "1");
        })
    }

    #[test]
    fn test_inject_globals_no_ravel_build_env_in_normal_mode() {
        with_ctx(|ctx| {
            Engine::inject_globals(&ctx, "", "", false, "/").unwrap();
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
                "/",
            )
            .unwrap();
            let filename: String = ctx.eval("__filename").unwrap();
            assert_eq!(filename, "/path/with \"quote/file.js");
        })
    }
}
