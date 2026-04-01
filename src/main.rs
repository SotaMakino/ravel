use std::env;
use std::fs;

use ravel::builtins::create_console;
use ravel::env::Env;
use ravel::interpreter::Interpreter;
use ravel::jsc::{evaluate_script, function_callback, JSContext, JSException, JSValue};
use ravel::lexer::lexer::Lexer;
use ravel::parser::parser::Parser;
use rustyline::DefaultEditor;

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
        println!("  --manual   Use manual interpreter backend");
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
        Backend::Manual => run_manual(source),
        Backend::JavaScriptCore => run_jsc(source),
    }
}

fn run_manual(source: &str) {
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
    let mut interp = Interpreter::new(&mut env);
    match interp.execute(&ast) {
        Ok(_) => {}
        Err(e) => eprintln!("Runtime error: {}", e),
    }
}

#[function_callback]
fn jsc_console_log(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    let parts: Vec<String> = arguments
        .iter()
        .filter_map(|v| v.as_string().ok().map(|s| s.to_string()))
        .collect();
    println!("{}", parts.join(" "));
    Ok(JSValue::new_undefined(ctx))
}

fn setup_jsc_console(ctx: &JSContext) -> JSValue {
    let obj = JSValue::new_array(ctx, &[]).unwrap().as_object().unwrap();
    obj.set_property(
        "log",
        JSValue::new_function(ctx, "log", Some(jsc_console_log)),
    )
    .unwrap();
    JSValue::from(obj)
}

fn run_jsc(source: &str) {
    let ctx = JSContext::default();
    let console = setup_jsc_console(&ctx);
    let global = ctx.global_object().unwrap();
    global.set_property("console", console).unwrap();

    match evaluate_script(&ctx, source, None, "ravel.js", 1) {
        Ok(val) => {
            if let Ok(s) = val.as_string() {
                println!("{}", s);
            }
        }
        Err(e) => eprintln!("JavaScriptCore error: {}", e),
    }
}

fn repl(backend: &Backend) {
    let mut rl = DefaultEditor::new().expect("Failed to initialize readline");

    let history_path = dirs::config_dir()
        .unwrap_or_default()
        .join("ravel")
        .join("history");
    let _ = rl.load_history(&history_path);

    match backend {
        Backend::Manual => {
            let mut env = Env::new();
            env.define("console", create_console());

            println!("ravel v0.2.0 (toy JS runtime) [manual backend]");

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
            }
        }
        Backend::JavaScriptCore => {
            let ctx = JSContext::default();
            let console = setup_jsc_console(&ctx);
            let global = ctx.global_object().unwrap();
            global.set_property("console", console).unwrap();

            println!("ravel v0.2.0 (toy JS runtime) [JavaScriptCore backend]");

            loop {
                let line = match rl.readline("> ") {
                    Ok(line) => line,
                    Err(_) => break,
                };

                let _ = rl.add_history_entry(&line);

                match evaluate_script(&ctx, line.as_str(), None, "repl.js", 1) {
                    Ok(val) => {
                        if let Ok(s) = val.as_string() {
                            println!("{}", s);
                        }
                    }
                    Err(e) => eprintln!("JavaScriptCore error: {}", e),
                }
            }
        }
    }

    let _ = rl.save_history(&history_path);
}
