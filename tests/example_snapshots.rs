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
