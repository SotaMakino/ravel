use rquickjs;
use rustyline::DefaultEditor;

use crate::console::value_to_string;
use crate::core::{Engine, finish_module, setup_module_loader, start_module};
use crate::error::{report_pending_rejections, report_uncaught};
use crate::transpiler::import_bindings;

/// Rewrite an import line so its bindings outlive it.
///
/// `import` only works in a module, and a module has its own scope, so the
/// names it binds are gone the moment the line finishes. Copying them onto
/// globalThis is what lets the next line still see them.
fn publish_bindings(line: &str, names: &[String]) -> String {
    let mut source = line.to_string();
    source.push('\n');
    for name in names {
        source.push_str(&format!("globalThis[{:?}] = {};\n", name, name));
    }
    source
}

pub fn repl(base: &str) {
    let mut rl = DefaultEditor::new().expect("Failed to initialize readline");

    let history_path = dirs::config_dir()
        .unwrap_or_default()
        .join("ravel")
        .join("history");
    let _ = rl.load_history(&history_path);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let engine = Engine::new().await;

        let cwd = std::env::current_dir().unwrap_or_default();
        let cwd_str = cwd.to_string_lossy().to_string();

        // Imports resolve from the working directory, so `import "./thing.js"`
        // means what it looks like it means.
        setup_module_loader(&engine.runtime, &cwd).await;

        rquickjs::async_with!(engine.context => |ctx| {
            if let Err(e) = Engine::setup_all_apis(&ctx, &cwd) {
                eprintln!("Environment setup error: {}", e);
            }
            if let Err(e) = Engine::inject_globals(&ctx, "", &cwd_str, false, base) {
                eprintln!("Global injection error: {}", e);
            }
        })
        .await;

        println!("ravel v{} (toy JS runtime)", crate::core::RAVEL_VERSION);

        // Each module needs a name of its own; reusing one would collide with
        // the module the previous line already registered under it.
        let mut line_number = 0usize;

        loop {
            let line = match rl.readline("> ") {
                Ok(line) => line,
                Err(_) => break,
            };

            let _ = rl.add_history_entry(&line);
            line_number += 1;

            let bindings = import_bindings(&line);

            // True only when a module was actually started, so a line that
            // failed to even resolve is not asked how it finished.
            let started = rquickjs::async_with!(engine.context => |ctx| {
                match &bindings {
                    Some(names) => {
                        let source = publish_bindings(&line, names);
                        let name = format!("<repl:{}>", line_number);
                        match start_module(&ctx, &source, &name) {
                            Ok(()) => true,
                            Err(e) => {
                                report_uncaught(&ctx, &e);
                                false
                            }
                        }
                    }
                    // Everything else stays a script, so `let` and `var` keep
                    // landing in the global scope and surviving the line.
                    None => {
                        match ctx.eval::<rquickjs::Value, _>(line.as_str()) {
                            Ok(val) => println!("{}", value_to_string(&val)),
                            Err(e) => report_uncaught(&ctx, &e),
                        }
                        false
                    }
                }
            })
            .await;

            engine.run_event_loop().await;

            if started {
                // A module only settles once the loop has drained, so its
                // failure cannot be reported before this point.
                let ok = rquickjs::async_with!(engine.context => |ctx| {
                    finish_module(&ctx)
                })
                .await;
                if ok {
                    println!("undefined");
                }
            }

            // Report and reset per line: the REPL keeps going either way, and
            // one line's rejection must not be blamed on the next.
            report_pending_rejections();
        }
    });

    let _ = rl.save_history(&history_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_bindings_copies_each_name() {
        let out = publish_bindings(
            r#"import { a, b } from "m";"#,
            &["a".to_string(), "b".to_string()],
        );
        assert!(out.starts_with(r#"import { a, b } from "m";"#));
        assert!(out.contains(r#"globalThis["a"] = a;"#));
        assert!(out.contains(r#"globalThis["b"] = b;"#));
    }

    #[test]
    fn test_publish_bindings_leaves_a_side_effect_import_alone() {
        let out = publish_bindings(r#"import "m";"#, &[]);
        assert_eq!(out.trim(), r#"import "m";"#);
    }

    #[test]
    fn test_publish_bindings_keeps_the_line_on_its_own_line() {
        // A trailing line comment would otherwise swallow what follows it.
        let out = publish_bindings(r#"import d from "m"; // note"#, &["d".to_string()]);
        assert!(out.contains("// note\nglobalThis"), "got: {}", out);
    }
}
