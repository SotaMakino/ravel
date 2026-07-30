use std::process::Command;

fn run_ravel(args: &[&str]) -> (String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_ravel"))
        .args(args)
        .output()
        .expect("Failed to execute ravel");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr)
}

#[test]
fn test_example_basic() {
    let (out, err) = run_ravel(&["examples/basic.js"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("basic", output);
}

#[test]
fn test_example_fs() {
    let (out, err) = run_ravel(&["examples/fs.js"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    let mut settings = insta::Settings::clone_current();
    settings.add_filter(
        r"/[^\s]*ravel/examples/fs\.js",
        "<PROJECT_ROOT>/examples/fs.js",
    );
    settings.add_filter(r"/[^\s]*ravel/examples", "<PROJECT_ROOT>/examples");
    let _guard = settings.bind_to_scope();
    insta::assert_snapshot!("fs", output);
}

#[test]
fn test_example_sandbox() {
    let (out, err) = run_ravel(&["examples/sandbox.js"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("sandbox", output);
}

#[test]
fn test_example_timers() {
    let (out, err) = run_ravel(&["examples/timers.js"]);
    let mut lines: Vec<&str> = out.lines().collect();
    lines.sort();
    let sorted_out = lines.join("\n");
    let output = format!(
        "=== STDOUT (sorted) ===\n{}\n=== STDERR ===\n{}",
        sorted_out, err
    );
    let mut settings = insta::Settings::clone_current();
    settings.add_filter(r"~\d+ms", "~<N>ms");
    let _guard = settings.bind_to_scope();
    insta::assert_snapshot!("timers", output);
}

#[test]
fn test_example_event_loop() {
    let (out, err) = run_ravel(&["examples/event-loop.js"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    // Only the order of the lines is fixed; the exact milliseconds are not.
    let mut settings = insta::Settings::clone_current();
    settings.add_filter(r"~\d+ms", "~<N>ms");
    let _guard = settings.bind_to_scope();
    insta::assert_snapshot!("event_loop", output);
}

#[test]
fn test_example_packages() {
    let (out, err) = run_ravel(&["examples/packages/main.js"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("packages", output);
}

#[test]
fn test_example_esm() {
    let (out, err) = run_ravel(&["examples/esm.js"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("esm", output);
}

#[test]
fn test_example_typescript() {
    let (out, err) = run_ravel(&["examples/typescript.ts"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("typescript", output);
}

#[test]
fn test_example_ts_esm() {
    let (out, err) = run_ravel(&["examples/ts-esm.ts"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("ts_esm", output);
}

#[test]
fn test_example_jsx() {
    let (out, err) = run_ravel(&["examples/jsx.tsx"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("jsx", output);
}

#[test]
fn test_example_encoding() {
    let (out, err) = run_ravel(&["examples/encoding.js"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("encoding", output);
    let _ = std::fs::remove_file("examples/encoding-out.txt");
    let _ = std::fs::remove_file("examples/encoding-out.bin");
}

#[test]
fn test_example_errors() {
    let (out, err) = run_ravel(&["examples/errors.js"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("errors", output);
}

#[test]
fn test_example_site_build() {
    let _ = std::fs::remove_dir_all("examples/site/dist");
    let _ = std::fs::remove_dir_all("dist");
    // No path filters: the build script no longer prints absolute paths.
    let (out, err) = run_ravel(&["--build", "examples/site/build.tsx"]);
    let output = format!("=== STDOUT ===\n{}\n=== STDERR ===\n{}", out, err);
    insta::assert_snapshot!("site_build", output);
    let _ = std::fs::remove_dir_all("examples/site/dist");
    let _ = std::fs::remove_dir_all("dist");
}
