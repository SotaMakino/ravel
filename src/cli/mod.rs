use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

use rquickjs::{AsyncContext, AsyncRuntime, Ctx, Result};
use rustyline::DefaultEditor;

use crate::timer::{TimerMessage, TimerState, get_timer_state, set_timer_state};
use crate::console::{value_to_string, setup_console};
use crate::fs::setup_fs;
use crate::timer::setup_timers;
use crate::core::{run_module, setup_module_loader};
use crate::transpiler::{is_typescript_file, transpile_ts};
use crate::jsx::setup_jsx_runtime;

const RAVEL_VERSION: &str = env!("CARGO_PKG_VERSION");

fn setup_all_apis<'js>(ctx: &Ctx<'js>, root: &Path) -> Result<()> {
    setup_console(ctx)?;
    setup_timers(ctx)?;
    setup_fs(ctx, root)?;
    setup_jsx_runtime(ctx)?;
    Ok(())
}

fn inject_globals<'js>(ctx: &Ctx<'js>, file: &str, dir: &str, build_mode: bool) -> Result<()> {
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

pub fn run(filename: &str) {
    let raw_source = fs::read_to_string(filename).expect("Failed to read file");

    let source = if is_typescript_file(filename) {
        match transpile_ts(&raw_source, filename) {
            Ok(js) => js,
            Err(e) => {
                eprintln!("TypeScript transpile error: {}", e);
                return;
            }
        }
    } else {
        raw_source
    };

    run_source(&source, filename);
}

pub fn run_source(source: &str, filename: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let ctx = AsyncContext::full(&runtime)
            .await
            .expect("Failed to create context");

        let (timer_state, mut timer_rx) = TimerState::new();
        set_timer_state(timer_state.clone());

        let abs_path = std::path::Path::new(filename)
            .canonicalize()
            .expect("Failed to resolve absolute path");
        let root = abs_path.parent().unwrap().to_path_buf();
        let dir = root.to_string_lossy().to_string();
        let file = abs_path.to_string_lossy().to_string();
        let file_name = abs_path.file_name().unwrap().to_string_lossy().to_string();

        let _prev_dir = std::env::current_dir().unwrap_or_default();
        std::env::set_current_dir(&root).expect("Failed to change directory");

        setup_module_loader(&runtime, &root).await;

        rquickjs::async_with!(ctx => |ctx| {
            if let Err(e) = setup_all_apis(&ctx, &root) {
                eprintln!("Environment setup error: {}", e);
            }
            if let Err(e) = inject_globals(&ctx, &file, &dir, false) {
                eprintln!("Global injection error: {}", e);
            }

            match run_module(&ctx, source, &file_name).await {
                Ok(_) => {}
                Err(e) => eprintln!("QuickJS error: {}", e),
            }
        })
        .await;

        loop {
            tokio::select! {
                Some(msg) = timer_rx.recv() => {
                    let ctx_clone = ctx.clone();
                    rquickjs::async_with!(ctx_clone => |ctx| {
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
    });
}

pub fn repl() {
    let mut rl = DefaultEditor::new().expect("Failed to initialize readline");

    let history_path = dirs::config_dir()
        .unwrap_or_default()
        .join("ravel")
        .join("history");
    let _ = rl.load_history(&history_path);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let ctx = AsyncContext::full(&runtime)
            .await
            .expect("Failed to create context");

        let (timer_state, mut timer_rx) = TimerState::new();
        set_timer_state(timer_state.clone());

        let cwd = std::env::current_dir().unwrap_or_default();
        let cwd_str = cwd.to_string_lossy().to_string();

        rquickjs::async_with!(ctx => |ctx| {
            if let Err(e) = setup_all_apis(&ctx, &cwd) {
                eprintln!("Environment setup error: {}", e);
            }
            if let Err(e) = inject_globals(&ctx, "", &cwd_str, false) {
                eprintln!("Global injection error: {}", e);
            }
        })
        .await;

        println!("ravel v{} (toy JS runtime)", RAVEL_VERSION);

        loop {
            let line = match rl.readline("> ") {
                Ok(line) => line,
                Err(_) => break,
            };

            let _ = rl.add_history_entry(&line);

            rquickjs::async_with!(ctx => |ctx| {
                match ctx.eval::<rquickjs::Value, _>(line.as_str()) {
                    Ok(val) => {
                        println!("{}", value_to_string(&val));
                    }
                    Err(e) => eprintln!("QuickJS error: {}", e),
                }
            })
            .await;

            loop {
                tokio::select! {
                    Some(msg) = timer_rx.recv() => {
                        let ctx_clone = ctx.clone();
                        rquickjs::async_with!(ctx_clone => |ctx| {
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
    });

    let _ = rl.save_history(&history_path);
}

pub fn print_help() {
    println!("Usage: ravel [OPTIONS] [FILE]");
    println!();
    println!("Options:");
    println!("  --help, -h       Show this help message");
    println!("  --version, -v    Show version information");
    println!("  --build <FILE>   Run script in SSG build mode (one-off, no timers)");
}

pub fn print_version() {
    println!("ravel v{}", RAVEL_VERSION);
}

pub fn build(filename: &str) {
    let raw_source = fs::read_to_string(filename).expect("Failed to read file");

    let source = if is_typescript_file(filename) {
        match transpile_ts(&raw_source, filename) {
            Ok(js) => js,
            Err(e) => {
                eprintln!("TypeScript transpile error: {}", e);
                return;
            }
        }
    } else {
        raw_source
    };

    build_source(&source, filename);
}

pub fn build_source(source: &str, filename: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let ctx = AsyncContext::full(&runtime)
            .await
            .expect("Failed to create context");

        let abs_path = std::path::Path::new(filename)
            .canonicalize()
            .expect("Failed to resolve absolute path");
        let root = abs_path.parent().unwrap().to_path_buf();
        let dir = root.to_string_lossy().to_string();
        let file = abs_path.to_string_lossy().to_string();
        let file_name = abs_path.file_name().unwrap().to_string_lossy().to_string();

        let _prev_dir = std::env::current_dir().unwrap_or_default();
        std::env::set_current_dir(&root).expect("Failed to change directory");

        setup_module_loader(&runtime, &root).await;

        rquickjs::async_with!(ctx => |ctx| {
            if let Err(e) = setup_all_apis(&ctx, &root) {
                eprintln!("Environment setup error: {}", e);
            }
            if let Err(e) = inject_globals(&ctx, &file, &dir, true) {
                eprintln!("Global injection error: {}", e);
            }

            match run_module(&ctx, source, &file_name).await {
                Ok(_) => {}
                Err(e) => eprintln!("QuickJS error: {}", e),
            }
        })
        .await;
    });
}
