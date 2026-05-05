mod build;
mod repl;
mod run;
mod serve;

pub use build::{build, build_source};
pub use repl::repl;
pub use run::{run, run_source};
pub use serve::serve;

use std::fs;

use crate::core::RAVEL_VERSION;
use crate::transpiler::{is_typescript_file, transpile_ts};

pub fn print_help() {
    println!("Usage: ravel [OPTIONS] [FILE]");
    println!();
    println!("Options:");
    println!("  --help, -h         Show this help message");
    println!("  --version, -v      Show version information");
    println!("  --serve [PORT]     Serve the dist/ directory (default port: 3000)");
    println!("  --base <PATH>      Base path prefix to strip when serving (e.g. /repo-name)");
    println!("  --build <FILE>     Run script in SSG build mode (one-off, no timers)");
    println!();
    println!("Config:");
    println!("  Reads ravel.json from CWD if present. CLI flags override config values.");
    println!("  Example ravel.json:");
    println!(r#"    {{ "base": "/repo-name", "port": 8080 }}"#);
}

pub fn print_version() {
    println!("ravel v{}", RAVEL_VERSION);
}

fn read_and_transpile(filename: &str) -> Option<String> {
    let raw_source = fs::read_to_string(filename).expect("Failed to read file");
    if is_typescript_file(filename) {
        match transpile_ts(&raw_source, filename) {
            Ok(js) => Some(js),
            Err(e) => {
                eprintln!("TypeScript transpile error: {}", e);
                None
            }
        }
    } else {
        Some(raw_source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_and_transpile_js_file() {
        let dir = std::env::temp_dir().join("ravel_test_read_js");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.js");
        std::fs::write(&file_path, "console.log(42);").unwrap();
        let result = read_and_transpile(file_path.to_str().unwrap());
        assert_eq!(result, Some("console.log(42);".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_and_transpile_ts_file() {
        let dir = std::env::temp_dir().join("ravel_test_read_ts");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.ts");
        std::fs::write(&file_path, "const x: number = 1;").unwrap();
        let result = read_and_transpile(file_path.to_str().unwrap());
        assert!(result.is_some());
        let transpiled = result.unwrap();
        assert!(!transpiled.contains("number"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}