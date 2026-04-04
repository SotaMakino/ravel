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
