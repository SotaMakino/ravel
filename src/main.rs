use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

use ravel::builtins::create_console;
use ravel::env::Env;
use ravel::interpreter::Interpreter;
use ravel::lexer::lexer::Lexer;
use ravel::parser::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let filename = &args[1];
        let source = fs::read_to_string(filename).expect("Failed to read file");
        run(&source);
        return;
    }

    repl();
}

fn run(source: &str) {
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

fn repl() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut env = Env::new();
    env.define("console", create_console());

    println!("ravel v0.1.0 (toy JS runtime)");

    for line in stdin.lock().lines() {
        print!("> ");
        stdout.flush().unwrap();

        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

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
