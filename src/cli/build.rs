use std::fs;


use rquickjs::{AsyncContext, AsyncRuntime};

use crate::core::{run_module, setup_module_loader};

use super::{inject_globals, read_and_transpile, setup_all_apis};

pub fn build(filename: &str) {
    let source = match read_and_transpile(filename) {
        Some(s) => s,
        None => return,
    };
    build_source(&source, filename);
}

pub fn build_source(source: &str, filename: &str) {
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    let abs_path = std::path::Path::new(filename)
        .canonicalize()
        .expect("Failed to resolve absolute path");
    let root = abs_path.parent().unwrap().to_path_buf();
    let script_dist = root.join("dist");

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let ctx = AsyncContext::full(&runtime)
            .await
            .expect("Failed to create context");

        let dir = root.to_string_lossy().to_string();
        let file = abs_path.to_string_lossy().to_string();
        let file_name = abs_path.file_name().unwrap().to_string_lossy().to_string();

        std::env::set_current_dir(&root).expect("Failed to change directory");

        setup_module_loader(&runtime, &root).await;

        rquickjs::async_with!(ctx => |ctx| {
            if let Err(e) = setup_all_apis(&ctx, &root) {
                eprintln!("Environment setup error: {}", e);
            }
            if let Err(e) = inject_globals(&ctx, &file, &dir, true) {
                eprintln!("Global injection error: {}", e);
            }

            match run_module(&ctx, source, &file_name).await {
                Ok(_) => {}
                Err(e) => eprintln!("QuickJS error: {}", e),
            }
        })
        .await;
    });

    if script_dist != original_dir.join("dist") && script_dist.exists() {
        let target = original_dir.join("dist");
        if target.exists() {
            let _ = fs::remove_dir_all(&target);
        }
        let _ = fs::create_dir_all(target.parent().unwrap());
        let _ = fs::rename(&script_dist, &target);
    }
}

