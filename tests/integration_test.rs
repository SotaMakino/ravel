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

fn run_file(name: &str, source: &str) -> (String, String) {
    let tmp = std::env::temp_dir().join(name);
    std::fs::write(&tmp, source).expect("Failed to write temp file");
    let result = run_ravel(&[tmp.to_str().unwrap()]);
    let _ = std::fs::remove_file(tmp);
    result
}

#[test]
fn test_hello_world() {
    let (out, err) = run_file("hello.js", "console.log(\"Hello, world!\");");
    assert_eq!(err, "");
    assert!(out.contains("Hello, world!"));
}

#[test]
fn test_arithmetic() {
    let (out, err) = run_file("arith.js", "console.log(1 + 2 * 3);");
    assert_eq!(err, "");
    assert!(out.contains("7"));
}

#[test]
fn test_variables() {
    let source = r#"
        let x = 10;
        let y = 20;
        console.log(x + y);
    "#;
    let (out, err) = run_file("vars.js", source);
    assert_eq!(err, "");
    assert!(out.contains("30"));
}

#[test]
fn test_if_else() {
    let source = r#"
        if (true) {
            console.log("yes");
        } else {
            console.log("no");
        }
    "#;
    let (out, err) = run_file("if.js", source);
    assert_eq!(err, "");
    assert!(out.contains("yes"));
    assert!(!out.contains("no"));
}

#[test]
fn test_while_loop() {
    let source = r#"
        let i = 0;
        while (i < 3) {
            console.log(i);
            i = i + 1;
        }
    "#;
    let (out, err) = run_file("while.js", source);
    assert_eq!(err, "");
    assert!(out.contains("0"));
    assert!(out.contains("1"));
    assert!(out.contains("2"));
}

#[test]
fn test_for_loop() {
    let source = r#"
        for (let i = 0; i < 3; i = i + 1) {
            console.log(i);
        }
    "#;
    let (out, err) = run_file("for.js", source);
    assert_eq!(err, "");
    assert!(out.contains("0"));
    assert!(out.contains("1"));
    assert!(out.contains("2"));
}

#[test]
fn test_function() {
    let source = r#"
        function add(a, b) {
            return a + b;
        }
        console.log(add(3, 4));
    "#;
    let (out, err) = run_file("func.js", source);
    assert_eq!(err, "");
    assert!(out.contains("7"));
}

#[test]
fn test_object() {
    let source = r#"
        let obj = { name: "test", value: 42 };
        console.log(obj.name);
        console.log(obj.value);
    "#;
    let (out, err) = run_file("obj.js", source);
    assert_eq!(err, "");
    assert!(out.contains("test"));
    assert!(out.contains("42"));
}

#[test]
fn test_array() {
    let source = r#"
        let arr = [1, 2, 3];
        console.log(arr[0]);
        console.log(arr[2]);
    "#;
    let (out, err) = run_file("arr.js", source);
    assert_eq!(err, "");
    assert!(out.contains("1"));
    assert!(out.contains("3"));
}

#[test]
fn test_string_concat() {
    let source = r#"
        console.log("Hello, " + "world!");
    "#;
    let (out, err) = run_file("concat.js", source);
    assert_eq!(err, "");
    assert!(out.contains("Hello, world!"));
}

#[test]
fn test_closure() {
    let source = r#"
        let x = 100;
        function get() {
            return x;
        }
        console.log(get());
    "#;
    let (out, err) = run_file("closure.js", source);
    assert_eq!(err, "");
    assert!(out.contains("100"));
}

#[test]
fn test_lexer_error() {
    let (_out, err) = run_file("error.js", "let x = \"unterminated");
    assert!(!err.is_empty());
    assert!(err.contains("Lexer error") || err.contains("Unterminated"));
}

#[test]
fn test_undefined_var_error() {
    let (_out, err) = run_file("undef.js", "console.log(missing);");
    assert!(!err.is_empty());
    assert!(err.contains("Undefined variable"));
}

#[test]
fn test_jsc_backend() {
    let (out, _err) = run_ravel(&["--jsc", "sample_jsc.js"]);
    assert!(out.contains("Arrow Functions"));
    assert!(out.contains("Template Literals"));
    assert!(out.contains("Array Methods"));
    assert!(out.contains("Math"));
    assert!(out.contains("Object Methods"));
    assert!(out.contains("String Methods"));
    assert!(out.contains("Destructuring"));
    assert!(out.contains("Classes"));
    assert!(out.contains("JSON"));
    assert!(out.contains("Try/Catch"));
    assert!(out.contains("Spread"));
    assert!(out.contains("All done!"));
}
