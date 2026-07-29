use rquickjs;
use rustyline::DefaultEditor;

use crate::console::value_to_string;
use crate::core::Engine;
use crate::error::{report_pending_rejections, report_uncaught};

pub fn repl() {
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

        rquickjs::async_with!(engine.context => |ctx| {
            if let Err(e) = Engine::setup_all_apis(&ctx, &cwd) {
                eprintln!("Environment setup error: {}", e);
            }
            if let Err(e) = Engine::inject_globals(&ctx, "", &cwd_str, false) {
                eprintln!("Global injection error: {}", e);
            }
        })
        .await;

        println!("ravel v{} (toy JS runtime)", crate::core::RAVEL_VERSION);

        loop {
            let line = match rl.readline("> ") {
                Ok(line) => line,
                Err(_) => break,
            };

            let _ = rl.add_history_entry(&line);

            rquickjs::async_with!(engine.context => |ctx| {
                match ctx.eval::<rquickjs::Value, _>(line.as_str()) {
                    Ok(val) => {
                        println!("{}", value_to_string(&val));
                    }
                    Err(e) => report_uncaught(&ctx, &e),
                }
            })
            .await;

            engine.run_event_loop().await;

            // Report and reset per line: the REPL keeps going either way, and
            // one line's rejection must not be blamed on the next.
            report_pending_rejections();
        }
    });

    let _ = rl.save_history(&history_path);
}