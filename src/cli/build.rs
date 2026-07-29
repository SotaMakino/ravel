use std::fs;

use crate::core::{Engine, finish_module, setup_module_loader, start_module};
use crate::error::{report_pending_rejections, report_uncaught};

use super::read_and_transpile;

/// Returns false if the build failed, so the caller can exit non-zero.
pub fn build(filename: &str) -> bool {
    let source = match read_and_transpile(filename) {
        Some(s) => s,
        None => return false,
    };
    build_source(&source, filename)
}

pub fn build_source(source: &str, filename: &str) -> bool {
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    let abs_path = std::path::Path::new(filename)
        .canonicalize()
        .expect("Failed to resolve absolute path");
    let root = abs_path.parent().unwrap().to_path_buf();
    let script_dist = root.join("dist");

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let ok = rt.block_on(async {
        let engine = Engine::new().await;

        let dir = root.to_string_lossy().to_string();
        let file = abs_path.to_string_lossy().to_string();
        let file_name = abs_path.file_name().unwrap().to_string_lossy().to_string();

        std::env::set_current_dir(&root).expect("Failed to change directory");

        setup_module_loader(&engine.runtime, &root).await;

        let ok = rquickjs::async_with!(engine.context => |ctx| {
            let mut ok = true;
            if let Err(e) = Engine::setup_all_apis(&ctx, &root) {
                eprintln!("Environment setup error: {}", e);
                ok = false;
            }
            if let Err(e) = Engine::inject_globals(&ctx, &file, &dir, true) {
                eprintln!("Global injection error: {}", e);
                ok = false;
            }

            if let Err(e) = start_module(&ctx, source, &file_name) {
                report_uncaught(&ctx, &e);
                ok = false;
            }
            ok
        })
        .await;

        // A build script is a script like any other: it can await a read or
        // set a timer, so it gets the same loop.
        let loop_ok = engine.run_event_loop().await;

        let module_ok = rquickjs::async_with!(engine.context => |ctx| {
            finish_module(&ctx)
        })
        .await;

        ok && loop_ok && module_ok && !report_pending_rejections()
    });

    if script_dist != original_dir.join("dist") && script_dist.exists() {
        let target = original_dir.join("dist");
        if target.exists() {
            let _ = fs::remove_dir_all(&target);
        }
        let _ = fs::create_dir_all(target.parent().unwrap());
        let _ = fs::rename(&script_dist, &target);
    }

    ok
}