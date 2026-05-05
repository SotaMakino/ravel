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
        fs.writeFile("nested/deep/file.txt", data);
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
    assert!(out.contains("wrote dist/index.html"));
    assert!(out.contains("wrote dist/about.html"));
    assert!(out.contains("wrote dist/style.css"));
    assert!(out.contains("wrote dist/blog/index.html"));
    assert!(out.contains("done"));

    let index = std::fs::read_to_string("dist/index.html").unwrap();
    assert!(index.contains("<title>Home</title>"));
    assert!(index.contains("<h1>Welcome</h1>"));

    let about = std::fs::read_to_string("dist/about.html").unwrap();
    assert!(about.contains("<title>About</title>"));
    assert!(about.contains("<h1>About</h1>"));

    let css = std::fs::read_to_string("dist/style.css").unwrap();
    assert!(css.contains("font-family"));

    let blog_index = std::fs::read_to_string("dist/blog/index.html").unwrap();
    assert!(blog_index.contains("<title>Blog</title>"));
    assert!(blog_index.contains("/ravel/style.css"));

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
