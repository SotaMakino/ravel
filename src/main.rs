use std::fs;
use std::time::Duration;

use rquickjs::{AsyncContext, AsyncRuntime};
use ravel::qjs::timers::{TimerMessage, TimerState, get_timer_state, set_timer_state};
use ravel::qjs::setup_full_environment;
use rustyline::DefaultEditor;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file_args: Vec<&String> = args
        .iter()
        .filter(|a| *a != "--help")
        .collect();

    if args.iter().any(|a| a == "--help") {
        println!("Usage: ravel [OPTIONS] [FILE]");
        println!();
        println!("Options:");
        println!("  --help     Show this help message");
        return;
    }

    if file_args.len() > 1 {
        let filename = file_args[1];
        let source = fs::read_to_string(filename).expect("Failed to read file");
        run_qjs(&source, filename);
        return;
    }

    repl_qjs();
}

fn run_qjs(source: &str, filename: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let ctx = AsyncContext::full(&runtime).await.expect("Failed to create context");

        let (timer_state, mut timer_rx) = TimerState::new();
        set_timer_state(timer_state.clone());

        let abs_path = std::path::Path::new(filename)
            .canonicalize()
            .expect("Failed to resolve absolute path");
        let root = abs_path.parent().unwrap().to_path_buf();
        let dir = root.to_string_lossy().to_string();
        let file = abs_path.to_string_lossy().to_string();

        rquickjs::async_with!(ctx => |ctx| {
            if let Err(e) = setup_full_environment(&ctx, &root) {
                eprintln!("Environment setup error: {}", e);
            }
            let _: Result<(), _> = ctx.eval(format!("var __filename = {:?};", file));
            let _: Result<(), _> = ctx.eval(format!("var __dirname = {:?};", dir));

            match ctx.eval::<rquickjs::Value, _>(source) {
                Ok(val) => {
                    if let Some(s) = val.as_string() {
                        if let Ok(string) = s.to_string() {
                            println!("{}", string);
                        }
                    }
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
                                let _: Result<(), _> = ctx.eval(format!("__ravel_fire_timer({})", id));
                                if let Some(state) = get_timer_state() {
                                    state.entries.lock().unwrap().remove(&id);
                                }
                            }
                            TimerMessage::FireInterval(id) => {
                                let _: Result<(), _> = ctx.eval(format!("__ravel_fire_interval({})", id));
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

fn repl_qjs() {
    let mut rl = DefaultEditor::new().expect("Failed to initialize readline");

    let history_path = dirs::config_dir()
        .unwrap_or_default()
        .join("ravel")
        .join("history");
    let _ = rl.load_history(&history_path);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let ctx = AsyncContext::full(&runtime).await.expect("Failed to create context");

        let (timer_state, mut timer_rx) = TimerState::new();
        set_timer_state(timer_state.clone());

        let cwd = std::env::current_dir().unwrap_or_default();

        rquickjs::async_with!(ctx => |ctx| {
            if let Err(e) = setup_full_environment(&ctx, &cwd) {
                eprintln!("Environment setup error: {}", e);
            }
        })
        .await;

        println!("ravel v0.3.0 (toy JS runtime)");

        loop {
            let line = match rl.readline("> ") {
                Ok(line) => line,
                Err(_) => break,
            };

            let _ = rl.add_history_entry(&line);

            rquickjs::async_with!(ctx => |ctx| {
                match ctx.eval::<rquickjs::Value, _>(line.as_str()) {
                    Ok(val) => {
                        if let Some(s) = val.as_string() {
                            if let Ok(string) = s.to_string() {
                                println!("{}", string);
                            }
                        }
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
                                    let _: Result<(), _> = ctx.eval(format!("__ravel_fire_timer({})", id));
                                    if let Some(state) = get_timer_state() {
                                        state.entries.lock().unwrap().remove(&id);
                                    }
                                }
                                TimerMessage::FireInterval(id) => {
                                    let _: Result<(), _> = ctx.eval(format!("__ravel_fire_interval({})", id));
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
