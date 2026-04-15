use std::path::Path;

use rquickjs::{AsyncContext, AsyncRuntime};

use crate::core::{run_module, setup_module_loader};
use crate::timer::{TimerState, set_timer_state};

use super::{drain_timers, inject_globals, read_and_transpile, setup_all_apis};

pub fn run(filename: &str) {
    let source = match read_and_transpile(filename) {
        Some(s) => s,
        None => return,
    };
    run_source(&source, filename);
}

pub fn run_source(source: &str, filename: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let ctx = AsyncContext::full(&runtime)
            .await
            .expect("Failed to create context");

        let (timer_state, mut timer_rx) = TimerState::new();
        set_timer_state(timer_state.clone());

        let abs_path = Path::new(filename)
            .canonicalize()
            .expect("Failed to resolve absolute path");
        let root = abs_path.parent().unwrap().to_path_buf();
        let dir = root.to_string_lossy().to_string();
        let file = abs_path.to_string_lossy().to_string();
        let file_name = abs_path.file_name().unwrap().to_string_lossy().to_string();

        let _prev_dir = std::env::current_dir().unwrap_or_default();
        std::env::set_current_dir(&root).expect("Failed to change directory");

        setup_module_loader(&runtime, &root).await;

        rquickjs::async_with!(ctx => |ctx| {
            if let Err(e) = setup_all_apis(&ctx, &root) {
                eprintln!("Environment setup error: {}", e);
            }
            if let Err(e) = inject_globals(&ctx, &file, &dir, false) {
                eprintln!("Global injection error: {}", e);
            }

            match run_module(&ctx, source, &file_name).await {
                Ok(_) => {}
                Err(e) => eprintln!("QuickJS error: {}", e),
            }
        })
        .await;

        drain_timers(&ctx, &mut timer_rx).await;
    });
}

