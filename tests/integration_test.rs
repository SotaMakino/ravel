use std::io::{Read, Write};
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

/// Like `run_ravel`, but also reports the process exit code.
fn run_ravel_status(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_ravel"))
        .args(args)
        .output()
        .expect("Failed to execute ravel");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.code().unwrap_or(-1))
}

fn run_file_status(name: &str, source: &str) -> (String, String, i32) {
    let tmp = std::env::temp_dir().join(name);
    std::fs::write(&tmp, source).expect("Failed to write temp file");
    let result = run_ravel_status(&[tmp.to_str().unwrap()]);
    let _ = std::fs::remove_file(tmp);
    result
}

fn run_file_with_deps(name: &str, source: &str, deps: &[(&str, &str)]) -> (String, String) {
    let dir = std::env::temp_dir().join(format!("ravel_test_{}", name));
    std::fs::create_dir_all(&dir).expect("Failed to create test dir");
    for (dep_name, dep_source) in deps {
        let dep_path = dir.join(dep_name);
        std::fs::write(&dep_path, dep_source).expect("Failed to write dep file");
    }
    let main_path = dir.join(name);
    std::fs::write(&main_path, source).expect("Failed to write main file");
    let result = run_ravel(&[main_path.to_str().unwrap()]);
    let _ = std::fs::remove_dir_all(&dir);
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
fn test_quickjs_backend() {
    let (out, _err) = run_ravel(&["examples/basic.js"]);
    assert!(out.contains("Variables"));
    assert!(out.contains("Functions"));
    assert!(out.contains("Control Flow"));
    assert!(out.contains("Loops"));
    assert!(out.contains("Arrays"));
    assert!(out.contains("Objects"));
    assert!(out.contains("Math"));
    assert!(out.contains("JSON"));
    assert!(out.contains("Done"));
}

#[test]
fn test_esm_named_import() {
    let (out, err) = run_file_with_deps(
        "esm_main.js",
        r#"
            import { add } from "./math.js";
            console.log(add(3, 4));
        "#,
        &[("math.js", "export function add(a, b) { return a + b; }")],
    );
    assert_eq!(err, "");
    assert!(out.contains("7"));
}

#[test]
fn test_esm_default_import() {
    let (out, err) = run_file_with_deps(
        "esm_default_main.js",
        r#"
            import greet from "./greet.js";
            console.log(greet("World"));
        "#,
        &[(
            "greet.js",
            "export default function(name) { return `Hello, ${name}!`; }",
        )],
    );
    assert_eq!(err, "");
    assert!(out.contains("Hello, World!"));
}

#[test]
fn test_esm_multiple_imports() {
    let (out, err) = run_file_with_deps(
        "esm_multi_main.js",
        r#"
            import { add, multiply } from "./ops.js";
            import { PI } from "./constants.js";
            console.log(add(2, 3));
            console.log(multiply(2, 3));
            console.log(PI);
        "#,
        &[
            ("ops.js", "export function add(a, b) { return a + b; } export function multiply(a, b) { return a * b; }"),
            ("constants.js", "export const PI = 3.14159;"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("5"));
    assert!(out.contains("6"));
    assert!(out.contains("3.14159"));
}

#[test]
fn test_ts_type_stripping() {
    let source = r#"
        const x: number = 42;
        console.log(x);
    "#;
    let (out, err) = run_file("types.ts", source);
    assert_eq!(err, "");
    assert!(out.contains("42"));
}

#[test]
fn test_ts_interface() {
    let source = r#"
        interface User {
            name: string;
            age: number;
        }
        const user: User = { name: "Alice", age: 30 };
        console.log(user.name);
    "#;
    let (out, err) = run_file("interface.ts", source);
    assert_eq!(err, "");
    assert!(out.contains("Alice"));
}

#[test]
fn test_ts_function_types() {
    let source = r#"
        function add(a: number, b: number): number {
            return a + b;
        }
        console.log(add(10, 20));
    "#;
    let (out, err) = run_file("fn_types.ts", source);
    assert_eq!(err, "");
    assert!(out.contains("30"));
}

#[test]
fn test_ts_enum() {
    let source = r#"
        enum Color { Red, Green, Blue }
        console.log(Color[Color.Green]);
    "#;
    let (out, err) = run_file("enum.ts", source);
    assert_eq!(err, "");
    assert!(out.contains("Green"));
}

#[test]
fn test_ts_class_types() {
    let source = r#"
        class Counter {
            count: number;
            constructor(initial: number) {
                this.count = initial;
            }
            increment(): number {
                this.count++;
                return this.count;
            }
        }
        const c: Counter = new Counter(0);
        c.increment();
        c.increment();
        console.log(c.count);
    "#;
    let (out, err) = run_file("class_types.ts", source);
    assert_eq!(err, "");
    assert!(out.contains("2"));
}

#[test]
fn test_ts_esm_imports() {
    let (out, err) = run_file_with_deps(
        "ts_main.ts",
        r#"
            import { add } from "./math.ts";
            const result: number = add(5, 3);
            console.log(result);
        "#,
        &[(
            "math.ts",
            "export function add(a: number, b: number): number { return a + b; }",
        )],
    );
    assert_eq!(err, "");
    assert!(out.contains("8"));
}

#[test]
fn test_ts_type_imports() {
    let source = r#"
        import type { Foo } from "./types.ts";
        const x: Foo = { bar: 42 };
        console.log(x.bar);
    "#;
    let (out, err) = run_file_with_deps(
        "type_import.ts",
        source,
        &[("types.ts", "export interface Foo { bar: number; }")],
    );
    assert_eq!(err, "");
    assert!(out.contains("42"));
}

#[test]
fn test_jsx_simple_element() {
    let source = r#"
        const el = <div>Hello</div>;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_simple.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("<div>Hello</div>"));
}

#[test]
fn test_jsx_with_attributes() {
    let source = r#"
        const el = <a href="https://example.com">Link</a>;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_attrs.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains(r#"<a href="https://example.com">Link</a>"#));
}

#[test]
fn test_jsx_nested_elements() {
    let source = r#"
        const el = <div><span>inner</span></div>;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_nested.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("<div><span>inner</span></div>"));
}

#[test]
fn test_jsx_self_closing() {
    let source = r#"
        const el = <br />;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_void.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("<br>"));
}

#[test]
fn test_jsx_fragment() {
    let source = r#"
        const el = <><div>A</div><div>B</div></>;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_frag.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("<div>A</div><div>B</div>"));
}

#[test]
fn test_jsx_function_component() {
    let source = r#"
        function Greeting(props) {
            return <span>Hello, {props.name}</span>;
        }
        const el = <Greeting name="World" />;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_component.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("<span>Hello, World</span>"));
}

#[test]
fn test_jsx_with_expression() {
    let source = r#"
        const name = "world";
        const el = <div>Hello {name}</div>;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_expr.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("<div>Hello world</div>"));
}

#[test]
fn test_jsx_full_page() {
    let source = r#"
        function Page() {
            return (
                <html>
                    <head>
                        <title>Test</title>
                    </head>
                    <body>
                        <h1>Welcome</h1>
                        <p>This is a test page.</p>
                    </body>
                </html>
            );
        }
        console.log(<Page />);
    "#;
    let (out, err) = run_file("jsx_page.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("<html>"));
    assert!(out.contains("<title>Test</title>"));
    assert!(out.contains("<h1>Welcome</h1>"));
    assert!(out.contains("<p>This is a test page.</p>"));
}

#[test]
fn test_jsx_xss_escapes_children() {
    let source = r#"
        const userInput = "<script>alert('xss')</script>";
        const el = <div>{userInput}</div>;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_xss_children.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("&lt;script&gt;"));
    assert!(out.contains("&lt;/script&gt;"));
    assert!(!out.contains("<script>"));
}

#[test]
fn test_jsx_xss_escapes_attribute_values() {
    let source = r#"
        const malicious = '"><script>alert(1)</script>';
        const el = <a href={malicious}>click</a>;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_xss_attr.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("&quot;&gt;&lt;script&gt;"));
    assert!(!out.contains("\"><script>"));
}

#[test]
fn test_jsx_xss_escapes_img_onerror() {
    let source = r#"
        const payload = "<img onerror=alert(1) src=x>";
        const el = <div>{payload}</div>;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_xss_img.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("&lt;img"));
    assert!(out.contains("&gt;"));
    assert!(!out.contains("<img"));
}

#[test]
fn test_jsx_xss_escapes_single_quotes_in_attr() {
    let source = r#"
        const val = "alert('xss')";
        const el = <div onclick={val}>test</div>;
        console.log(el);
    "#;
    let (out, err) = run_file("jsx_xss_singlequote.tsx", source);
    assert_eq!(err, "");
    assert!(out.contains("&#x27;"));
    assert!(!out.contains("'xss'"));
}

#[test]
fn test_build_flag_requires_file() {
    let (out, err) = run_ravel(&["--build"]);
    assert!(!out.contains("Hello"));
    assert!(err.contains("--build requires a script file"));
}

#[test]
fn test_build_mode_sets_ravel_build_global() {
    let source = r#"
        console.log(ravel.build);
    "#;
    let tmp = std::env::temp_dir().join("build_test.js");
    std::fs::write(&tmp, source).expect("Failed to write temp file");
    let (out, err) = run_ravel(&["--build", tmp.to_str().unwrap()]);
    let _ = std::fs::remove_file(tmp);
    assert_eq!(err, "");
    assert!(out.contains("true"));
}

#[test]
fn test_normal_mode_ravel_build_is_false() {
    let (out, err) = run_file("normal_build.js", "console.log(ravel.build);");
    assert_eq!(err, "");
    assert!(out.contains("false"));
}

#[test]
fn test_ravel_version() {
    let (out, err) = run_file("version.js", "console.log(ravel.version);");
    assert_eq!(err, "");
    assert!(out.contains("0.3.0"));
}

#[test]
fn test_process_env() {
    let (out, err) = run_file("env.js", "console.log(typeof process.env);");
    assert_eq!(err, "");
    assert!(out.contains("object"));
}

#[test]
fn test_process_env_has_path() {
    let (out, err) = run_file(
        "env_path.js",
        "console.log(typeof process.env.PATH !== 'undefined');",
    );
    assert_eq!(err, "");
    assert!(out.contains("true"));
}

#[test]
fn test_build_mode_env_has_ravel_build() {
    let source = r#"
        console.log(process.env.RAVEL_BUILD);
    "#;
    let tmp = std::env::temp_dir().join("build_env_test.js");
    std::fs::write(&tmp, source).expect("Failed to write temp file");
    let (out, err) = run_ravel(&["--build", tmp.to_str().unwrap()]);
    let _ = std::fs::remove_file(tmp);
    assert_eq!(err, "");
    assert!(out.contains("1"));
}

#[test]
fn test_normal_mode_env_no_ravel_build() {
    let source = r#"
        console.log(process.env.RAVEL_BUILD);
    "#;
    let (out, err) = run_file("no_build_env.js", source);
    assert_eq!(err, "");
    assert!(out.contains("undefined"));
}

#[test]
fn test_fs_mkdir_sync() {
    let source = r#"
        fs.mkdirSync("test_dir");
        console.log(fs.exists("test_dir"));
    "#;
    let (out, err) = run_file("mkdir.js", source);
    assert_eq!(err, "");
    assert!(out.contains("true"));
}

#[test]
fn test_fs_mkdir_sync_nested() {
    let source = r#"
        fs.mkdirSync("a/b/c");
        console.log(fs.exists("a/b/c"));
    "#;
    let (out, err) = run_file("mkdir_nested.js", source);
    assert_eq!(err, "");
    assert!(out.contains("true"));
}

#[test]
fn test_fs_write_file_creates_directories() {
    let source = r#"
        var data = new Uint8Array([72, 101, 108, 108, 111]);
        fs.writeFileSync("nested/deep/file.txt", data);
        console.log(fs.exists("nested/deep/file.txt"));
    "#;
    let (out, err) = run_file("write_nested.js", source);
    assert_eq!(err, "");
    assert!(out.contains("true"));
}

#[test]
fn test_fs_mkdir_sync_escape_attempt() {
    let source = r#"
        try {
            fs.mkdirSync("../escape_dir");
            console.log("escaped");
        } catch (e) {
            console.log("blocked");
        }
    "#;
    let (out, err) = run_file("mkdir_escape.js", source);
    assert_eq!(err, "");
    assert!(out.contains("blocked"));
}

#[test]
fn test_version_flag() {
    let (out, err) = run_ravel(&["--version"]);
    assert_eq!(err, "");
    assert!(out.contains("0.3.0"));
}

#[test]
fn test_short_version_flag() {
    let (out, err) = run_ravel(&["-v"]);
    assert_eq!(err, "");
    assert!(out.contains("0.3.0"));
}

#[test]
fn test_short_help_flag() {
    let (out, err) = run_ravel(&["-h"]);
    assert_eq!(err, "");
    assert!(out.contains("--build"));
}

#[test]
fn test_example_site_build() {
    let _ = std::fs::remove_dir_all("examples/site/dist");
    let _ = std::fs::remove_dir_all("dist");
    let (out, err) = run_ravel(&["--build", "examples/site/build.tsx"]);
    assert_eq!(err, "");
    for expected in [
        "wrote dist/index.html",
        "wrote dist/style.css",
        "wrote dist/posts.json",
        "wrote dist/app.js",
        "done",
    ] {
        assert!(out.contains(expected), "missing {:?} in: {}", expected, out);
    }

    let index = std::fs::read_to_string("dist/index.html").unwrap();
    assert!(index.contains("<title>ravel</title>"));
    assert!(index.contains(r#"<div id="app">"#));
    assert!(index.contains("/ravel/style.css"));

    // The import map has to survive with real quotes: escaping it to &quot;
    // would leave the browser unable to resolve "preact" at all.
    assert!(
        index.contains(r#"<script type="importmap">{"imports":{"preact":"#),
        "import map was escaped or missing: {}",
        index
    );
    // The `*` marks esm.sh deps external so htm shares one preact instance.
    assert!(index.contains(r#""htm/preact":"https://esm.sh/*htm@"#));
    assert!(!index.contains("&quot;"), "import map got HTML-escaped");

    let app = std::fs::read_to_string("dist/app.js").unwrap();
    assert!(app.contains(r#"from "preact""#), "app.js was not copied intact");

    let posts = std::fs::read_to_string("dist/posts.json").unwrap();
    assert_eq!(posts.matches(r#""slug""#).count(), 3, "posts.json: {}", posts);

    let css = std::fs::read_to_string("dist/style.css").unwrap();
    assert!(css.contains("font-family"));

    let _ = std::fs::remove_dir_all("dist");
}

#[test]
fn test_serve_no_dist_directory() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ravel"))
        .args(["--serve"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute ravel");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(stderr.contains("dist/ directory not found"));
}

#[test]
fn test_serve_serves_files() {
    let dir = tempfile::tempdir().unwrap();
    let dist_dir = dir.path().join("dist");
    std::fs::create_dir_all(&dist_dir).unwrap();
    std::fs::write(dist_dir.join("index.html"), "<h1>Hello</h1>").unwrap();

    let port = 18765u16;
    let mut child = Command::new(env!("CARGO_BIN_EXE_ravel"))
        .args(["--serve", &port.to_string()])
        .current_dir(dir.path())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start ravel");

    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut stream = std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        std::time::Duration::from_secs(2),
    )
    .expect("Failed to connect to server");

    stream
        .write_all(b"GET /index.html HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();
    let response_str = String::from_utf8_lossy(&response);

    child.kill().unwrap();
    let _ = child.wait();

    assert!(response_str.contains("<h1>Hello</h1>"));
}

#[test]
fn test_serve_default_port() {
    let (out, _err) = run_ravel(&["--help"]);
    assert!(out.contains("--serve [PORT]"));
}

// --- Error reporting and exit codes ---

#[test]
fn test_uncaught_error_reports_message_and_exits_nonzero() {
    let (_out, err, code) = run_file_status("err_msg.js", "null.x;");
    assert!(
        err.contains("Uncaught TypeError:") && err.contains("null"),
        "stderr was: {}",
        err
    );
    assert_eq!(code, 1);
}

#[test]
fn test_uncaught_error_includes_stack_frames() {
    let (_out, err, code) = run_file_status(
        "err_stack.js",
        "function a(){ null.x }\nfunction b(){ a() }\nb();",
    );
    assert!(err.contains("    at a ("), "stderr was: {}", err);
    assert!(err.contains("    at b ("), "stderr was: {}", err);
    assert_eq!(code, 1);
}

#[test]
fn test_uncaught_error_reports_line_number() {
    let (_out, err, code) = run_file_status("err_line.js", "console.log(1);\nnull.x;");
    assert!(err.contains(":2:"), "expected line 2 in stderr: {}", err);
    assert_eq!(code, 1);
}

#[test]
fn test_thrown_non_error_value_is_reported() {
    let (_out, err, code) = run_file_status("err_string.js", "throw 'boom';");
    assert!(err.contains("Uncaught boom"), "stderr was: {}", err);
    assert_eq!(code, 1);
}

#[test]
fn test_custom_error_name_is_preserved() {
    let (_out, err, code) = run_file_status("err_range.js", "throw new RangeError('too big');");
    assert!(
        err.contains("Uncaught RangeError: too big"),
        "stderr was: {}",
        err
    );
    assert_eq!(code, 1);
}

#[test]
fn test_successful_script_exits_zero() {
    let (out, err, code) = run_file_status("ok_exit.js", "console.log('fine');");
    assert!(out.contains("fine"));
    assert_eq!(err, "");
    assert_eq!(code, 0);
}

#[test]
fn test_missing_file_reports_error_and_exits_nonzero() {
    let (_out, err, code) = run_ravel_status(&["definitely_not_a_file.js"]);
    assert!(err.contains("cannot read"), "stderr was: {}", err);
    assert_eq!(code, 1);
}

#[test]
fn test_unhandled_rejection_is_reported() {
    let (_out, err, code) = run_file_status(
        "rej_unhandled.js",
        "Promise.reject(new Error('nope'));\nconsole.log('after');",
    );
    assert!(
        err.contains("Unhandled promise rejection: Error: nope"),
        "stderr was: {}",
        err
    );
    assert_eq!(code, 1);
}

#[test]
fn test_handled_rejection_is_not_reported() {
    let (out, err, code) = run_file_status(
        "rej_handled.js",
        "Promise.reject(new Error('x')).catch(() => console.log('caught'));",
    );
    assert!(out.contains("caught"));
    assert_eq!(err, "");
    assert_eq!(code, 0);
}

#[test]
fn test_throw_inside_then_becomes_unhandled_rejection() {
    let (_out, err, code) = run_file_status(
        "rej_then.js",
        "Promise.resolve().then(() => { throw new Error('in then'); });",
    );
    assert!(
        err.contains("Unhandled promise rejection: Error: in then"),
        "stderr was: {}",
        err
    );
    assert_eq!(code, 1);
}

#[test]
fn test_error_in_timer_callback_exits_nonzero() {
    let (_out, err, code) = run_file_status("err_timer.js", "setTimeout(() => { null.x; }, 1);");
    assert!(err.contains("Uncaught TypeError:"), "stderr was: {}", err);
    assert_eq!(code, 1);
}

#[test]
fn test_timer_stack_hides_ravel_internals() {
    let (_out, err, _code) = run_file_status("err_timer2.js", "setTimeout(() => { null.x; }, 1);");
    assert!(!err.contains("__ravel_"), "internal frames leaked: {}", err);
}

#[test]
fn test_failed_build_exits_nonzero() {
    let dir = std::env::temp_dir().join("ravel_test_build_fail");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("build.js");
    std::fs::write(&script, "throw new Error('build broke');").unwrap();
    let (_out, err, code) = run_ravel_status(&["--build", script.to_str().unwrap()]);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        err.contains("Uncaught Error: build broke"),
        "stderr was: {}",
        err
    );
    assert_eq!(code, 1);
}

// --- Microtasks and async continuations ---

#[test]
fn test_microtasks_run_before_exit() {
    let (out, err, code) = run_file_status(
        "micro.js",
        "Promise.resolve().then(() => console.log('microtask'));\nconsole.log('sync');",
    );
    assert!(out.contains("microtask"), "stdout was: {}", out);
    assert_eq!(err, "");
    assert_eq!(code, 0);
}

#[test]
fn test_await_continuation_runs() {
    let (out, _err, code) = run_file_status(
        "await.js",
        "async function f(){ await null; console.log('after await'); }\nf();",
    );
    assert!(out.contains("after await"), "stdout was: {}", out);
    assert_eq!(code, 0);
}

#[test]
fn test_awaited_rejection_can_be_caught() {
    let (out, err, code) = run_file_status(
        "await_catch.js",
        "async function f(){ try { await Promise.reject(new Error('r')); } catch(e) { console.log('caught:', e.message); } }\nf();",
    );
    assert!(out.contains("caught: r"), "stdout was: {}", out);
    assert_eq!(err, "");
    assert_eq!(code, 0);
}

// --- TextEncoder / TextDecoder ---

#[test]
fn test_text_encoder_encodes_utf8() {
    let (out, err) = run_file(
        "enc.js",
        "console.log(new TextEncoder().encode('abc').length);",
    );
    assert_eq!(err, "");
    assert!(out.contains("3"));
}

#[test]
fn test_text_encoder_counts_multibyte_as_bytes() {
    let (out, err) = run_file(
        "enc_multi.js",
        "console.log(new TextEncoder().encode('日本').length);",
    );
    assert_eq!(err, "");
    assert!(out.contains("6"));
}

#[test]
fn test_text_decoder_round_trip() {
    let (out, err) = run_file(
        "dec.js",
        "console.log(new TextDecoder().decode(new TextEncoder().encode('héllo 世界')));",
    );
    assert_eq!(err, "");
    assert!(out.contains("héllo 世界"));
}

#[test]
fn test_text_decoder_rejects_unsupported_encoding() {
    let (_out, err, code) = run_file_status("dec_bad.js", "new TextDecoder('latin1');");
    assert!(err.contains("RangeError"), "stderr was: {}", err);
    assert_eq!(code, 1);
}

#[test]
fn test_readme_ssg_example_runs() {
    // The README's build-mode example, which used to throw on TextEncoder.
    let dir = std::env::temp_dir().join("ravel_test_readme_ssg");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("build.js");
    std::fs::write(
        &script,
        r#"await fs.writeFile("dist/index.html", new TextEncoder().encode("<h1>Built</h1>"));"#,
    )
    .unwrap();
    let (_out, err, code) = run_ravel_status(&["--build", script.to_str().unwrap()]);
    assert_eq!(err, "");
    assert_eq!(code, 0);
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all("dist");
}

// --- fs.writeFile string support ---

#[test]
fn test_write_file_accepts_string() {
    let (out, err) = run_file(
        "write_str.js",
        r#"await fs.writeFile("ws.txt", "plain string");
           console.log(new TextDecoder().decode(await fs.readFile("ws.txt")));"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("plain string"));
}

#[test]
fn test_write_file_still_accepts_bytes() {
    let (out, err) = run_file(
        "write_bytes.js",
        r#"await fs.writeFile("wb.txt", new TextEncoder().encode("from bytes"));
           console.log(new TextDecoder().decode(await fs.readFile("wb.txt")));"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("from bytes"));
}

#[test]
fn test_write_file_rejects_other_types() {
    let (_out, err, code) = run_file_status("write_bad.js", r#"await fs.writeFile("bad.txt", {});"#);
    assert!(err.contains("Uncaught"), "stderr was: {}", err);
    assert_eq!(code, 1);
}

// --- console streams ---

#[test]
fn test_console_error_goes_to_stderr() {
    let (out, err) = run_file("con_err.js", "console.error('to stderr');");
    assert!(err.contains("to stderr"), "stderr was: {}", err);
    assert!(!out.contains("to stderr"), "leaked to stdout: {}", out);
}

#[test]
fn test_console_warn_goes_to_stderr() {
    let (out, err) = run_file("con_warn.js", "console.warn('warned');");
    assert!(err.contains("warned"), "stderr was: {}", err);
    assert!(!out.contains("warned"));
}

#[test]
fn test_console_info_and_debug_go_to_stdout() {
    let (out, err) = run_file("con_info.js", "console.info('i');\nconsole.debug('d');");
    assert_eq!(err, "");
    assert!(out.contains("i") && out.contains("d"));
}

// --- process globals ---

#[test]
fn test_process_argv_includes_script_path() {
    let (out, err) = run_file(
        "argv.js",
        "console.log(process.argv[1].endsWith('argv.js'));",
    );
    assert_eq!(err, "");
    assert!(out.contains("true"));
}

#[test]
fn test_process_argv_passes_user_args() {
    let tmp = std::env::temp_dir().join("argv_user.js");
    std::fs::write(&tmp, "console.log(process.argv.slice(2).join(','));").unwrap();
    let (out, err) = run_ravel(&[tmp.to_str().unwrap(), "alpha", "beta"]);
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(err, "");
    assert!(out.contains("alpha,beta"), "stdout was: {}", out);
}

#[test]
fn test_process_exit_sets_status_code() {
    let (out, _err, code) = run_file_status(
        "exit.js",
        "console.log('before');\nprocess.exit(3);\nconsole.log('never');",
    );
    assert!(out.contains("before"));
    assert!(!out.contains("never"));
    assert_eq!(code, 3);
}

#[test]
fn test_process_exit_defaults_to_zero() {
    let (_out, _err, code) = run_file_status("exit0.js", "process.exit();");
    assert_eq!(code, 0);
}

#[test]
fn test_process_env_survives_quotes_in_values() {
    let tmp = std::env::temp_dir().join("env_quote.js");
    std::fs::write(&tmp, "console.log(process.env.RAVEL_TEST_VAR);").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ravel"))
        .arg(tmp.to_str().unwrap())
        .env("RAVEL_TEST_VAR", r#"has "quotes" and \backslash"#)
        .output()
        .expect("Failed to execute ravel");
    let _ = std::fs::remove_file(&tmp);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains(r#"has "quotes" and \backslash"#), "stdout was: {}", out);
}

// --- Async fs ---

#[test]
fn test_read_file_returns_a_promise() {
    let (out, err) = run_file(
        "async_shape.js",
        r#"fs.writeFileSync("shape.txt", "x");
           console.log(fs.readFile("shape.txt").constructor.name);"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("Promise"), "stdout was: {}", out);
}

#[test]
fn test_read_file_resolves_after_synchronous_code() {
    // The promise must not settle before the module body finishes, or the
    // read was really synchronous.
    let (out, err) = run_file(
        "async_order.js",
        r#"fs.writeFileSync("order.txt", "data");
           fs.readFile("order.txt").then(() => console.log("2 async"));
           console.log("1 sync");"#,
    );
    assert_eq!(err, "");
    let sync_at = out.find("1 sync").expect("missing sync line");
    let async_at = out.find("2 async").expect("missing async line");
    assert!(sync_at < async_at, "wrong order: {}", out);
}

#[test]
fn test_pending_read_is_not_abandoned_at_exit() {
    // Regression: the runtime used to exit while the read was in flight.
    let (out, err) = run_file(
        "async_exit.js",
        r#"fs.writeFileSync("exit.txt", "kept");
           fs.readFile("exit.txt").then(b =>
             console.log("resolved:", new TextDecoder().decode(b)));"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("resolved: kept"), "stdout was: {}", out);
}

#[test]
fn test_await_read_file() {
    let (out, err) = run_file(
        "async_await.js",
        r#"fs.writeFileSync("await.txt", "awaited");
           const b = await fs.readFile("await.txt");
           console.log(new TextDecoder().decode(b));"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("awaited"));
}

#[test]
fn test_concurrent_reads() {
    let (out, err) = run_file(
        "async_all.js",
        r#"fs.writeFileSync("c.txt", "c");
           const all = await Promise.all([
             fs.readFile("c.txt"), fs.readFile("c.txt"), fs.readFile("c.txt"),
           ]);
           console.log("count:", all.length);"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("count: 3"));
}

#[test]
fn test_read_file_rejects_for_missing_file() {
    let (out, err) = run_file(
        "async_missing.js",
        r#"try { await fs.readFile("does_not_exist.txt"); }
           catch (e) { console.log("caught:", e.name); }"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("caught: Error"), "stdout was: {}", out);
}

#[test]
fn test_async_read_enforces_sandbox() {
    let (out, err) = run_file(
        "async_sandbox.js",
        r#"try { await fs.readFile("/etc/passwd"); console.log("LEAK"); }
           catch (e) { console.log("blocked"); }"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("blocked"), "stdout was: {}", out);
    assert!(!out.contains("LEAK"));
}

#[test]
fn test_async_write_enforces_sandbox() {
    let (out, err) = run_file(
        "async_sandbox_w.js",
        r#"try { await fs.writeFile("../escaped.txt", "x"); console.log("LEAK"); }
           catch (e) { console.log("blocked"); }"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("blocked"), "stdout was: {}", out);
}

#[test]
fn test_unhandled_read_rejection_is_reported() {
    let (_out, err, code) = run_file_status("async_unhandled.js", r#"fs.readFile("nope.txt");"#);
    assert!(
        err.contains("Unhandled promise rejection"),
        "stderr was: {}",
        err
    );
    assert_eq!(code, 1);
}

#[test]
fn test_write_file_returns_a_promise() {
    let (out, err) = run_file(
        "async_write.js",
        r#"await fs.writeFile("aw.txt", "written");
           console.log(new TextDecoder().decode(fs.readFileSync("aw.txt")));"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("written"));
}

#[test]
fn test_sync_variants_still_work() {
    let (out, err) = run_file(
        "sync_variants.js",
        r#"fs.writeFileSync("s.txt", "sync data");
           console.log(new TextDecoder().decode(fs.readFileSync("s.txt")));"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("sync data"));
}

#[test]
fn test_timers_still_run_after_awaited_io() {
    let (out, err) = run_file(
        "async_timer.js",
        r#"fs.writeFileSync("t.txt", "t");
           setTimeout(() => console.log("timer"), 1);
           await fs.readFile("t.txt");
           console.log("read");"#,
    );
    assert_eq!(err, "");
    assert!(out.contains("timer"), "timer never fired: {}", out);
    assert!(out.contains("read"));
}

// --- The event loop ---

#[test]
fn test_top_level_await_on_a_timer_resolves() {
    // The module parks on a promise that only a timer can settle. Firing that
    // timer is the loop's job, so this deadlocks unless the loop keeps running
    // while the module is parked.
    let (out, err, code) = run_file_status(
        "loop_sleep.js",
        r#"const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
           console.log("before");
           await sleep(10);
           console.log("after");"#,
    );
    assert_eq!(err, "");
    let before = out.find("before").expect("missing before");
    let after = out.find("after").expect("module never resumed");
    assert!(before < after, "wrong order: {}", out);
    assert_eq!(code, 0);
}

#[test]
fn test_timers_fire_while_the_module_is_parked_on_an_await() {
    let (out, err) = run_file(
        "loop_parked.js",
        r#"const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
           setTimeout(() => console.log("timer"), 20);
           await sleep(60);
           console.log("resumed");"#,
    );
    assert_eq!(err, "");
    let timer = out.find("timer").expect("timer never fired");
    let resumed = out.find("resumed").expect("module never resumed");
    assert!(timer < resumed, "the timer waited for the module: {}", out);
}

#[cfg(unix)]
#[test]
fn test_timers_fire_while_a_read_is_in_flight() {
    // A FIFO gives a read that stays in flight for a known length of time,
    // which a local file cannot. Timers must not wait for it.
    let dir = std::env::temp_dir().join("ravel_test_loop_fifo");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let fifo = dir.join("slow.pipe");
    let made = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("failed to run mkfifo");
    assert!(made.success(), "mkfifo failed");

    let script = dir.join("fifo.js");
    std::fs::write(
        &script,
        r#"setTimeout(() => console.log("timer"), 20);
           const bytes = await fs.readFile("slow.pipe");
           console.log("read:", new TextDecoder().decode(bytes));"#,
    )
    .unwrap();

    // Opening the FIFO for writing blocks until the reader arrives, so hand
    // the write to a helper and let the script get there on its own.
    let writer = std::thread::spawn({
        let fifo = fifo.clone();
        move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            std::fs::write(&fifo, b"piped").unwrap();
        }
    });

    let (out, err) = run_ravel(&[script.to_str().unwrap()]);
    writer.join().unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(err, "");
    let timer = out.find("timer").expect("timer never fired");
    let read = out.find("read: piped").expect("read never finished");
    assert!(timer < read, "the timer waited for the read: {}", out);
}

#[test]
fn test_timers_fire_in_deadline_order_not_registration_order() {
    let (out, err) = run_file(
        "loop_order.js",
        r#"setTimeout(() => console.log("third"), 60);
           setTimeout(() => console.log("first"), 20);
           setTimeout(() => console.log("second"), 40);"#,
    );
    assert_eq!(err, "");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["first", "second", "third"], "out was: {}", out);
}

#[test]
fn test_equal_deadlines_fire_in_registration_order() {
    let (out, err) = run_file(
        "loop_ties.js",
        r#"for (let i = 1; i <= 5; i++) setTimeout(() => console.log(i), 0);"#,
    );
    assert_eq!(err, "");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["1", "2", "3", "4", "5"], "out was: {}", out);
}

#[test]
fn test_microtasks_drain_between_timer_callbacks() {
    let (out, err) = run_file(
        "loop_micro.js",
        r#"setTimeout(() => {
             console.log("timer 1");
             Promise.resolve().then(() => console.log("microtask"));
           }, 10);
           setTimeout(() => console.log("timer 2"), 30);"#,
    );
    assert_eq!(err, "");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines,
        vec!["timer 1", "microtask", "timer 2"],
        "out was: {}",
        out
    );
}

#[test]
fn test_interval_survives_an_awaited_read_between_ticks() {
    let (out, err) = run_file(
        "loop_interval_io.js",
        r#"fs.writeFileSync("i.txt", "x");
           let ticks = 0;
           await new Promise((resolve) => {
             const id = setInterval(async () => {
               await fs.readFile("i.txt");
               ticks += 1;
               console.log("tick", ticks);
               if (ticks === 3) { clearInterval(id); resolve(); }
             }, 10);
           });
           console.log("cleared");"#,
    );
    assert_eq!(err, "");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines,
        vec!["tick 1", "tick 2", "tick 3", "cleared"],
        "out was: {}",
        out
    );
}

#[test]
fn test_timers_wake_on_their_deadline_not_on_a_tick() {
    // Fifty 1ms timers, each scheduled by the one before it. The loop sleeps
    // on each deadline in turn, so this costs about 50ms. A loop that instead
    // woke every 10ms to check the clock would take ten times as long, which
    // is the gap this threshold sits in.
    let (out, err) = run_file(
        "loop_chain.js",
        r#"const t0 = Date.now();
           let n = 0;
           const step = () => {
             n += 1;
             if (n < 50) { setTimeout(step, 1); return; }
             console.log(Date.now() - t0);
           };
           setTimeout(step, 1);"#,
    );
    assert_eq!(err, "");
    let elapsed: u64 = out.trim().parse().unwrap_or_else(|_| panic!("out: {}", out));
    assert!(
        elapsed < 300,
        "50 chained 1ms timers took {}ms; the loop is ticking, not sleeping on deadlines",
        elapsed
    );
}

// --- Module resolution ---

/// Build a project tree from `(relative path, contents)` pairs, run the entry
/// file, and clean up. The entry is always `main.js`.
fn run_project(name: &str, files: &[(&str, &str)]) -> (String, String, i32) {
    let dir = std::env::temp_dir().join(format!("ravel_resolve_{}", name));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, contents) in files {
        let full = dir.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, contents).unwrap();
    }
    let result = run_ravel_status(&[dir.join("main.js").to_str().unwrap()]);
    let _ = std::fs::remove_dir_all(&dir);
    result
}

#[test]
fn test_bare_import_resolves_from_node_modules() {
    let (out, err, code) = run_project(
        "bare",
        &[
            ("main.js", r#"import { v } from "dep"; console.log(v);"#),
            ("node_modules/dep/package.json", r#"{"main": "./lib.js"}"#),
            ("node_modules/dep/lib.js", "export const v = 'from dep';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("from dep"), "stdout was: {}", out);
    assert_eq!(code, 0);
}

#[test]
fn test_bare_import_walks_up_the_tree() {
    let (out, err, _) = run_project(
        "walkup",
        &[
            ("main.js", r#"import "./nested/deep/leaf.js";"#),
            (
                "nested/deep/leaf.js",
                r#"import { v } from "dep"; console.log(v);"#,
            ),
            ("node_modules/dep/index.js", "export const v = 'found above';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("found above"), "stdout was: {}", out);
}

#[test]
fn test_nearest_node_modules_copy_wins() {
    let (out, err, _) = run_project(
        "nearest",
        &[
            ("main.js", r#"import "./app/use.js";"#),
            ("app/use.js", r#"import { v } from "dep"; console.log(v);"#),
            ("node_modules/dep/index.js", "export const v = 'outer';"),
            ("app/node_modules/dep/index.js", "export const v = 'inner';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("inner"), "stdout was: {}", out);
    assert!(!out.contains("outer"));
}

#[test]
fn test_scoped_package_with_pattern_exports() {
    let (out, err, _) = run_project(
        "scoped",
        &[
            (
                "main.js",
                r#"import { v } from "@scope/pkg/thing"; console.log(v);"#,
            ),
            (
                "node_modules/@scope/pkg/package.json",
                r#"{"exports": {"./*": "./src/*.js"}}"#,
            ),
            ("node_modules/@scope/pkg/src/thing.js", "export const v = 'patterned';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("patterned"), "stdout was: {}", out);
}

#[test]
fn test_conditional_exports_pick_the_import_entry() {
    let (out, err, _) = run_project(
        "conditions",
        &[
            ("main.js", r#"import { v } from "dep"; console.log(v);"#),
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"require": "./cjs.js", "import": "./esm.js", "default": "./d.js"}}"#,
            ),
            ("node_modules/dep/cjs.js", "export const v = 'cjs';"),
            ("node_modules/dep/esm.js", "export const v = 'esm';"),
            ("node_modules/dep/d.js", "export const v = 'default';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("esm"), "stdout was: {}", out);
}

#[test]
fn test_exports_encapsulates_unlisted_files() {
    let (_out, err, code) = run_project(
        "encapsulation",
        &[
            ("main.js", r#"import "dep/private.js";"#),
            (
                "node_modules/dep/package.json",
                r#"{"exports": {".": "./main.js"}}"#,
            ),
            ("node_modules/dep/main.js", ""),
            ("node_modules/dep/private.js", "export const leaked = true;"),
        ],
    );
    assert!(err.contains("is not exported"), "stderr was: {}", err);
    assert_eq!(code, 1);
}

#[test]
fn test_imports_map_resolves_hash_specifiers() {
    let (out, err, _) = run_project(
        "imports",
        &[
            ("main.js", r##"import { v } from "#internal"; console.log(v);"##),
            ("package.json", r##"{"imports": {"#internal": "./src/thing.js"}}"##),
            ("src/thing.js", "export const v = 'via imports map';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("via imports map"), "stdout was: {}", out);
}

#[test]
fn test_imports_map_can_alias_a_package() {
    let (out, err, _) = run_project(
        "imports_alias",
        &[
            ("main.js", r##"import { v } from "#dep"; console.log(v);"##),
            ("package.json", r##"{"imports": {"#dep": "real"}}"##),
            ("node_modules/real/index.js", "export const v = 'aliased';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("aliased"), "stdout was: {}", out);
}

#[test]
fn test_module_field_is_preferred_over_main() {
    // The real-world pairing: main is CommonJS, module is the ESM build.
    // Picking main hands ravel a file it cannot load at all, so this fails
    // outright rather than merely picking the less good entry point.
    let (out, err, code) = run_project(
        "module_field",
        &[
            ("main.js", r#"import { v } from "dep"; console.log(v);"#),
            (
                "node_modules/dep/package.json",
                r#"{"main": "./cjs.js", "module": "./esm.js"}"#,
            ),
            ("node_modules/dep/cjs.js", "module.exports = { v: 'cjs' };"),
            ("node_modules/dep/esm.js", "export const v = 'esm';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("esm"), "stdout was: {}", out);
    assert_eq!(code, 0);
}

#[test]
fn test_extensionless_relative_import_finds_typescript() {
    // Used to be a documented limit: ./helper never found helper.ts.
    let (out, err, _) = run_project(
        "ts_ext",
        &[
            ("main.js", r#"import { v } from "./helper"; console.log(v);"#),
            ("helper.ts", "export const v: string = 'typescript';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("typescript"), "stdout was: {}", out);
}

#[test]
fn test_directory_import_uses_index() {
    let (out, err, _) = run_project(
        "dir_index",
        &[
            ("main.js", r#"import { v } from "./utils"; console.log(v);"#),
            ("utils/index.js", "export const v = 'index';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("index"), "stdout was: {}", out);
}

#[test]
fn test_missing_package_names_itself_in_the_error() {
    let (_out, err, code) = run_project(
        "missing_pkg",
        &[("main.js", r#"import "no-such-package";"#)],
    );
    assert!(
        err.contains("cannot find package 'no-such-package'"),
        "stderr was: {}",
        err
    );
    assert_eq!(code, 1);
}

#[test]
fn test_exports_target_cannot_escape_its_package() {
    let (_out, err, code) = run_project(
        "escape",
        &[
            ("main.js", r#"import "dep/../../outside";"#),
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"./*": "./*.js"}}"#,
            ),
            ("outside.js", "export const leaked = true;"),
        ],
    );
    assert!(
        err.contains("escapes its package"),
        "stderr was: {}",
        err
    );
    assert_eq!(code, 1);
}

#[test]
fn test_one_file_imported_two_ways_is_one_module() {
    // Module state must not be duplicated when the same file is reached by
    // two different specifiers.
    let (out, err, _) = run_project(
        "identity",
        &[
            (
                "main.js",
                r#"import { bump, count } from "./state.js";
                   import { bump as bump2 } from "./nested/../state.js";
                   bump(); bump2();
                   console.log("count:", count());"#,
            ),
            (
                "state.js",
                "let n = 0; export const bump = () => { n += 1; }; export const count = () => n;",
            ),
            ("nested/keep.js", ""),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("count: 2"), "stdout was: {}", out);
}

// --- REPL ---

/// Drive the REPL by piping lines to its stdin, in a directory of our making.
fn run_repl(name: &str, lines: &str, files: &[(&str, &str)]) -> (String, String) {
    let dir = std::env::temp_dir().join(format!("ravel_repl_{}", name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, contents) in files {
        let full = dir.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, contents).unwrap();
    }

    let mut child = Command::new(env!("CARGO_BIN_EXE_ravel"))
        .current_dir(&dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start the REPL");
    child
        .stdin
        .as_mut()
        .expect("no stdin")
        .write_all(lines.as_bytes())
        .expect("Failed to write to the REPL");
    // Closing stdin is what ends the session.
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("REPL did not exit");
    let _ = std::fs::remove_dir_all(&dir);
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn test_repl_imports_a_local_module() {
    let (out, err) = run_repl(
        "local",
        "import { v } from \"./lib.js\"\nv\n",
        &[("lib.js", "export const v = 'imported';")],
    );
    assert_eq!(err, "");
    assert!(out.contains("imported"), "stdout was: {}", out);
}

#[test]
fn test_repl_import_bindings_survive_to_later_lines() {
    // The point of the whole exercise: a module's scope ends with the line,
    // so the binding has to be republished to still be there afterwards.
    let (out, err) = run_repl(
        "persist",
        "import { add } from \"./m.js\"\nadd(2, 3)\nadd(10, 5)\n",
        &[("m.js", "export const add = (a, b) => a + b;")],
    );
    assert_eq!(err, "");
    assert!(out.contains("5"), "stdout was: {}", out);
    assert!(out.contains("15"), "stdout was: {}", out);
}

#[test]
fn test_repl_import_forms() {
    let (out, err) = run_repl(
        "forms",
        "import d from \"./m.js\"\nd\nimport { a as b } from \"./m.js\"\nb\nimport * as ns from \"./m.js\"\ntypeof ns.a\n",
        &[(
            "m.js",
            "export default 'the-default'; export const a = 'aliased';",
        )],
    );
    assert_eq!(err, "");
    assert!(out.contains("the-default"), "default import: {}", out);
    assert!(out.contains("aliased"), "aliased import: {}", out);
    assert!(out.contains("string"), "namespace import: {}", out);
}

#[test]
fn test_repl_bare_import_resolves_from_node_modules() {
    let (out, err) = run_repl(
        "bare",
        "import { v } from \"dep\"\nv\n",
        &[
            ("node_modules/dep/package.json", r#"{"main": "./i.js"}"#),
            ("node_modules/dep/i.js", "export const v = 'from dep';"),
        ],
    );
    assert_eq!(err, "");
    assert!(out.contains("from dep"), "stdout was: {}", out);
}

#[test]
fn test_repl_survives_a_failed_import() {
    let (out, err) = run_repl("recover", "import { x } from \"./gone.js\"\n42\n", &[]);
    assert!(err.contains("Error resolving module"), "stderr: {}", err);
    assert!(out.contains("42"), "the REPL stopped after the error: {}", out);
    // A line that never started must not also claim to have finished.
    assert!(!out.contains("undefined"), "spurious result line: {}", out);
}

#[test]
fn test_repl_dynamic_import_works() {
    let (out, err) = run_repl(
        "dynamic",
        "import(\"./m.js\").then(m => console.log(\"got\", m.v))\n",
        &[("m.js", "export const v = 'dynamic';")],
    );
    assert_eq!(err, "");
    assert!(out.contains("got dynamic"), "stdout was: {}", out);
}

#[test]
fn test_repl_declarations_still_persist() {
    // Non-import lines stay scripts, so this must not have regressed.
    let (out, err) = run_repl("decls", "var a = 1\nlet b = 2\nconst c = 3\na + b + c\n", &[]);
    assert_eq!(err, "");
    assert!(out.contains("6"), "stdout was: {}", out);
}

#[test]
fn test_repl_module_is_evaluated_once() {
    let (out, err) = run_repl(
        "cache",
        "import { v } from \"./m.js\"\nimport { v as w } from \"./m.js\"\nw\n",
        &[(
            "m.js",
            "console.log('side effect'); export const v = 'cached';",
        )],
    );
    assert_eq!(err, "");
    assert_eq!(
        out.matches("side effect").count(),
        1,
        "module body ran more than once: {}",
        out
    );
    assert!(out.contains("cached"));
}

// --- ravel.base ---

/// Run a script with the working directory set to its own project, so
/// `ravel.json` is found the way it would be in a real project.
fn run_in_project(name: &str, files: &[(&str, &str)]) -> (String, String) {
    let dir = std::env::temp_dir().join(format!("ravel_base_{}", name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, contents) in files {
        std::fs::write(dir.join(path), contents).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_ravel"))
        .current_dir(&dir)
        .arg("main.js")
        .output()
        .expect("Failed to execute ravel");
    let _ = std::fs::remove_dir_all(&dir);
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn test_ravel_base_comes_from_the_config_file() {
    let (out, err) = run_in_project(
        "configured",
        &[
            ("ravel.json", r#"{"base": "/my-repo"}"#),
            ("main.js", "console.log(ravel.base);"),
        ],
    );
    assert_eq!(err, "");
    assert_eq!(out.trim(), "/my-repo/");
}

#[test]
fn test_ravel_base_defaults_to_root() {
    let (out, err) = run_in_project("default", &[("main.js", "console.log(ravel.base);")]);
    assert_eq!(err, "");
    assert_eq!(out.trim(), "/");
}

#[test]
fn test_ravel_base_always_ends_in_a_slash() {
    // Written without one; a <base href> lacking it resolves against the
    // parent directory, so normalising here is what keeps links correct.
    let (out, err) = run_in_project(
        "slash",
        &[
            ("ravel.json", r#"{"base": "/no-slash"}"#),
            ("main.js", "console.log(JSON.stringify(ravel.base));"),
        ],
    );
    assert_eq!(err, "");
    assert_eq!(out.trim(), "\"/no-slash/\"");
}

#[test]
fn test_site_build_uses_the_configured_base() {
    // The regression this whole change is about: the build script and the
    // server used to keep separate copies of the base and could drift.
    let dir = std::env::temp_dir().join("ravel_base_site");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("ravel.json"), r#"{"base": "/elsewhere"}"#).unwrap();
    std::fs::write(
        dir.join("build.tsx"),
        r#"fs.writeFileSync("dist/out.txt", ravel.base);"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ravel"))
        .current_dir(&dir)
        .args(["--build", "build.tsx"])
        .output()
        .expect("Failed to execute ravel");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written = std::fs::read_to_string(dir.join("dist/out.txt")).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(written, "/elsewhere/");
}
