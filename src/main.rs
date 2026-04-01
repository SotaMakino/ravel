use std::env;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

use javascriptcore::JSContext;
use ravel::jsc::timers::{JscTimerBridge, set_timer_bridge};
use ravel::jsc::setup_full_environment;
use rustyline::DefaultEditor;

#[cfg(feature = "manual")]
use ravel::builtins::{create_console, create_timer_globals};
#[cfg(feature = "manual")]
use ravel::env::Env;
#[cfg(feature = "manual")]
use ravel::interpreter::Interpreter;
#[cfg(feature = "manual")]
use ravel::lexer::lexer::Lexer;
#[cfg(feature = "manual")]
use ravel::parser::parser::Parser;
#[cfg(feature = "manual")]
use ravel::timer::TimerState;
#[cfg(feature = "manual")]
use tokio::runtime::Runtime;

#[derive(Debug, PartialEq)]
enum Backend {
    Manual,
    JavaScriptCore,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let backend = if args.iter().any(|a| a == "--manual") {
        Backend::Manual
    } else {
        Backend::JavaScriptCore
    };

    let file_args: Vec<&String> = args
        .iter()
        .filter(|a| *a != "--jsc" && *a != "--manual" && *a != "--help")
        .collect();

    if args.iter().any(|a| a == "--help") {
        println!("Usage: ravel [OPTIONS] [FILE]");
        println!();
        println!("Options:");
        println!("  --jsc      Use JavaScriptCore backend (default)");
        println!("  --manual   Use manual interpreter backend (experimental)");
        println!("  --help     Show this help message");
        return;
    }

    if file_args.len() > 1 {
        let filename = file_args[1];
        let source = fs::read_to_string(filename).expect("Failed to read file");
        run(&source, &backend);
        return;
    }

    repl(&backend);
}

fn run(source: &str, backend: &Backend) {
    match backend {
        #[cfg(feature = "manual")]
        Backend::Manual => run_manual(source),
        #[cfg(not(feature = "manual"))]
        Backend::Manual => {
            eprintln!("Manual backend not compiled. Rebuild with: cargo build --features manual");
        }
        Backend::JavaScriptCore => run_jsc(source),
    }
}

fn run_jsc(source: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let handle = rt.handle().clone();

    rt.block_on(async {
        let ctx = JSContext::default();

        let timer_bridge = Arc::new(Mutex::new(JscTimerBridge::new(ctx, handle.clone())));
        set_timer_bridge(timer_bridge.clone());

        {
            let bridge = timer_bridge.lock().unwrap();
            match setup_full_environment(&bridge.ctx) {
                Ok(_) => {}
                Err(e) => eprintln!("Environment setup error: {}", e),
            }
        }

        let eval_result = {
            let bridge = timer_bridge.lock().unwrap();
            javascriptcore::evaluate_script(&bridge.ctx, source, None, "ravel.js", 1)
        };
        match eval_result {
            Ok(val) => {
                if let Ok(s) = val.as_string() {
                    println!("{}", s);
                }
            }
            Err(e) => eprintln!("JavaScriptCore error: {}", e),
        }

        let timer_state = timer_bridge.lock().unwrap().state.clone();
        while timer_state.has_pending() {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    });
}

#[cfg(feature = "manual")]
fn run_manual(source: &str) {
    let rt = Runtime::new().expect("Failed to create tokio runtime");
    let handle = rt.handle().clone();

    let lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            return;
        }
    };

    let parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            return;
        }
    };

    let mut env = Env::new();
    env.define("console", create_console());

    let timer_state = TimerState::new();
    for (name, value) in create_timer_globals(timer_state.clone(), handle.clone(), &mut env as *mut Env) {
        env.define(&name, value);
    }

    let mut interp = Interpreter::new(&mut env);
    match interp.execute(&ast) {
        Ok(_) => {}
        Err(e) => eprintln!("Runtime error: {}", e),
    }

    rt.block_on(async {
        while timer_state.has_pending() {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    });
}

fn repl(backend: &Backend) {
    let mut rl = DefaultEditor::new().expect("Failed to initialize readline");

    let history_path = dirs::config_dir()
        .unwrap_or_default()
        .join("ravel")
        .join("history");
    let _ = rl.load_history(&history_path);

    match backend {
        #[cfg(feature = "manual")]
        Backend::Manual => repl_manual(&mut rl, &history_path),
        #[cfg(not(feature = "manual"))]
        Backend::Manual => {
            eprintln!("Manual backend not compiled. Rebuild with: cargo build --features manual");
        }
        Backend::JavaScriptCore => repl_jsc(&mut rl, &history_path),
    }
}

#[cfg(feature = "manual")]
fn repl_manual(rl: &mut DefaultEditor, history_path: &std::path::Path) {
    let rt = Runtime::new().expect("Failed to create tokio runtime");
    let handle = rt.handle().clone();

    let mut env = Env::new();
    env.define("console", create_console());

    let timer_state = TimerState::new();
    for (name, value) in create_timer_globals(timer_state.clone(), handle.clone(), &mut env as *mut Env) {
        env.define(&name, value);
    }

    println!("ravel v0.3.0 (toy JS runtime) [manual backend]");

    loop {
        let line = match rl.readline("> ") {
            Ok(line) => line,
            Err(_) => break,
        };

        let _ = rl.add_history_entry(&line);

        let lexer = Lexer::new(&line);
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => {
                eprintln!("Lexer error: {}", e);
                continue;
            }
        };

        let parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parser error: {}", e);
                continue;
            }
        };

        let mut interp = Interpreter::new(&mut env);
        match interp.execute(&ast) {
            Ok(val) => println!("{}", val),
            Err(e) => eprintln!("Runtime error: {}", e),
        }

        rt.block_on(async {
            while timer_state.has_pending() {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        });
    }

    let _ = rl.save_history(history_path);
}

fn repl_jsc(rl: &mut DefaultEditor, history_path: &std::path::Path) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let handle = rt.handle().clone();

    rt.block_on(async {
        let ctx = JSContext::default();

        let timer_bridge = Arc::new(Mutex::new(JscTimerBridge::new(ctx, handle.clone())));
        set_timer_bridge(timer_bridge.clone());

        {
            let bridge = timer_bridge.lock().unwrap();
            match setup_full_environment(&bridge.ctx) {
                Ok(_) => {}
                Err(e) => eprintln!("Environment setup error: {}", e),
            }
        }

        println!("ravel v0.3.0 (toy JS runtime) [JavaScriptCore backend]");

        loop {
            let line = match rl.readline("> ") {
                Ok(line) => line,
                Err(_) => break,
            };

            let _ = rl.add_history_entry(&line);

            let eval_result = {
                let bridge = timer_bridge.lock().unwrap();
                javascriptcore::evaluate_script(&bridge.ctx, line.as_str(), None, "repl.js", 1)
            };
            match eval_result {
                Ok(val) => {
                    if let Ok(s) = val.as_string() {
                        println!("{}", s);
                    }
                }
                Err(e) => eprintln!("JavaScriptCore error: {}", e),
            }

            let timer_state = timer_bridge.lock().unwrap().state.clone();
            while timer_state.has_pending() {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }
    });

    let _ = rl.save_history(history_path);
}
