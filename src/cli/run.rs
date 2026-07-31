use std::path::Path;

use crate::core::{Engine, finish_module, setup_module_loader, start_module};
use crate::error::{report_pending_rejections, report_uncaught};

use super::read_and_transpile;

/// Returns false if the script failed, so the caller can exit non-zero.
pub fn run(filename: &str, base: &str) -> bool {
    let source = match read_and_transpile(filename) {
        Some(s) => s,
        None => return false,
    };
    run_source(&source, filename, base)
}

pub fn run_source(source: &str, filename: &str, base: &str) -> bool {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let engine = Engine::new().await;

        let abs_path = Path::new(filename)
            .canonicalize()
            .expect("Failed to resolve absolute path");
        let root = abs_path.parent().unwrap().to_path_buf();
        let dir = root.to_string_lossy().to_string();
        let file = abs_path.to_string_lossy().to_string();
        let file_name = abs_path.file_name().unwrap().to_string_lossy().to_string();

        let _prev_dir = std::env::current_dir().unwrap_or_default();
        std::env::set_current_dir(&root).expect("Failed to change directory");

        setup_module_loader(&engine.runtime, &root).await;

        let ok = rquickjs::async_with!(engine.context => |ctx| {
            let mut ok = true;
            if let Err(e) = Engine::setup_all_apis(&ctx, &root) {
                eprintln!("Environment setup error: {}", e);
                ok = false;
            }
            if let Err(e) = Engine::inject_globals(&ctx, &file, &dir, false, base) {
                eprintln!("Global injection error: {}", e);
                ok = false;
            }

            // Starts the module and returns at its first `await`; the event
            // loop below carries it the rest of the way.
            if let Err(e) = start_module(&ctx, source, &file_name) {
                report_uncaught(&ctx, &e);
                ok = false;
            }
            ok
        })
        .await;

        let loop_ok = engine.run_event_loop().await;

        // The loop has drained, so the module has settled if it ever will.
        let module_ok = rquickjs::async_with!(engine.context => |ctx| {
            finish_module(&ctx)
        })
        .await;

        // Timer and I/O callbacks can reject too, so check after the loop.
        ok && loop_ok && module_ok && !report_pending_rejections()
    })
}