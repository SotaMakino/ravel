use std::path::Path;

use crate::core::{Engine, run_module, setup_module_loader};
use crate::error::{report_pending_rejections, report_uncaught};

use super::read_and_transpile;

/// Returns false if the script failed, so the caller can exit non-zero.
pub fn run(filename: &str) -> bool {
    let source = match read_and_transpile(filename) {
        Some(s) => s,
        None => return false,
    };
    run_source(&source, filename)
}

pub fn run_source(source: &str, filename: &str) -> bool {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let mut engine = Engine::new().await;

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
            if let Err(e) = Engine::inject_globals(&ctx, &file, &dir, false) {
                eprintln!("Global injection error: {}", e);
                ok = false;
            }

            if let Err(e) = run_module(&ctx, source, &file_name).await {
                report_uncaught(&ctx, &e);
                ok = false;
            }
            ok
        })
        .await;

        // Settle promise callbacks queued by the module body before deciding
        // whether a rejection went unhandled.
        let jobs_ok = engine.run_pending_jobs().await;
        let timers_ok = engine.drain_timers().await;

        // Timer callbacks can reject too, so check after the event loop.
        ok && jobs_ok && timers_ok && !report_pending_rejections()
    })
}