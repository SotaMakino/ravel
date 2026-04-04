use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{JsxOptions, JsxRuntime, TransformOptions, Transformer};
use std::path::Path;

pub fn transpile_ts(source: &str, filename: &str) -> Result<String, String> {
    let allocator = Allocator::default();
    let source_type = if filename.ends_with(".tsx") {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };

    let parsed = Parser::new(&allocator, source, source_type).parse();

    if !parsed.errors.is_empty() {
        let msgs: Vec<String> = parsed.errors.iter().map(|e| format!("{e}")).collect();
        return Err(msgs.join("\n"));
    }

    let mut program = parsed.program;

    let semantic = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .build(&program);

    let mut options = TransformOptions::default();
    if filename.ends_with(".tsx") {
        options.jsx = JsxOptions {
            jsx_plugin: true,
            runtime: JsxRuntime::Classic,
            pragma: Some("note".to_string()),
            pragma_frag: Some("note".to_string()),
            throw_if_namespace: false,
            pure: false,
            ..Default::default()
        };
    }

    let ret = Transformer::new(&allocator, Path::new(filename), &options)
        .build_with_scoping(semantic.semantic.into_scoping(), &mut program);

    if !ret.errors.is_empty() {
        let msgs: Vec<String> = ret.errors.iter().map(|e| format!("{e}")).collect();
        return Err(msgs.join("\n"));
    }

    let output = Codegen::new().build(&program).code;
    Ok(output)
}

pub fn is_typescript_file(filename: &str) -> bool {
    filename.ends_with(".ts") || filename.ends_with(".tsx")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_type_annotation() {
        let input = "const x: number = 42;";
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("const x = 42"));
        assert!(!output.contains(": number"));
    }

    #[test]
    fn test_strip_interface() {
        let input = r#"
            interface User {
                name: string;
                age: number;
            }
            const user: User = { name: "Alice", age: 30 };
        "#;
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("const user = {"));
        assert!(!output.contains("interface"));
    }

    #[test]
    fn test_strip_function_types() {
        let input = "function add(a: number, b: number): number { return a + b; }";
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("function add(a, b)"));
        assert!(!output.contains(": number"));
    }

    #[test]
    fn test_strip_type_imports() {
        let input = r#"import type { Foo } from "./foo"; const x: Foo = {}; "#;
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(!output.contains("import type"));
    }

    #[test]
    fn test_strip_enum() {
        let input = "enum Color { Red, Green, Blue }";
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("Color"));
    }

    #[test]
    fn test_strip_class_types() {
        let input = r#"
            class Person {
                name: string;
                age: number;
                constructor(name: string, age: number) {
                    this.name = name;
                    this.age = age;
                }
            }
        "#;
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("class Person"));
        assert!(!output.contains(": string"));
        assert!(!output.contains(": number"));
    }

    #[test]
    fn test_strip_return_type() {
        let input = "const greet = (name: string): string => `Hello, ${name}`;";
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("=>"));
        assert!(!output.contains(": string"));
    }

    #[test]
    fn test_strip_generic_types() {
        let input = "const arr: Array<string> = [];";
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("const arr = []"));
        assert!(!output.contains("<string>"));
    }

    #[test]
    fn test_strip_type_assertion() {
        let input = r#"const el = document.getElementById("app") as HTMLElement;"#;
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("document.getElementById"));
        assert!(!output.contains("as HTMLElement"));
    }

    #[test]
    fn test_strip_namespace() {
        let input = r#"
            namespace Utils {
                export const VERSION = "1.0.0";
            }
        "#;
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("VERSION"));
    }

    #[test]
    fn test_tsx_file_detection() {
        let input = r#"const App = (): {name: string} => ({ name: "test" });"#;
        let output = transpile_ts(input, "test.tsx").unwrap();
        assert!(output.contains("const App ="));
        assert!(!output.contains(": {name: string}"));
    }

    #[test]
    fn test_is_typescript_file() {
        assert!(is_typescript_file("foo.ts"));
        assert!(is_typescript_file("bar.tsx"));
        assert!(!is_typescript_file("baz.js"));
        assert!(!is_typescript_file("qux.mjs"));
    }

    #[test]
    fn test_transpile_error() {
        let input = "const x: = 42;";
        let result = transpile_ts(input, "test.ts");
        assert!(result.is_err());
    }

    #[test]
    fn test_preserves_esm_imports() {
        let input = r#"import { foo } from "./foo"; console.log(foo);"#;
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("import { foo }"));
        assert!(output.contains("console.log(foo)"));
    }

    #[test]
    fn test_preserves_esm_exports() {
        let input = r#"export const x = 42; export default function() {}"#;
        let output = transpile_ts(input, "test.ts").unwrap();
        assert!(output.contains("export const x = 42"));
        assert!(output.contains("export default"));
    }

    #[test]
    fn test_jsx_simple_element() {
        let input = r#"const el = <div>Hello</div>;"#;
        let output = transpile_ts(input, "test.tsx").unwrap();
        assert!(output.contains(r#"note("div""#));
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_jsx_with_attributes() {
        let input = r#"const el = <a href="https://example.com">Link</a>;"#;
        let output = transpile_ts(input, "test.tsx").unwrap();
        assert!(output.contains(r#"note("a""#));
        assert!(output.contains(r#"href"#));
    }

    #[test]
    fn test_jsx_nested_elements() {
        let input = r#"const el = <div><span>inner</span></div>;"#;
        let output = transpile_ts(input, "test.tsx").unwrap();
        assert!(output.contains(r#"note("div""#));
        assert!(output.contains(r#"note("span""#));
    }

    #[test]
    fn test_jsx_self_closing() {
        let input = r#"const el = <br />;"#;
        let output = transpile_ts(input, "test.tsx").unwrap();
        assert!(output.contains(r#"note("br""#));
    }

    #[test]
    fn test_jsx_with_expression() {
        let input = r#"const name = "world"; const el = <div>Hello {name}</div>;"#;
        let output = transpile_ts(input, "test.tsx").unwrap();
        assert!(output.contains(r#"note("div""#));
        assert!(output.contains("name"));
    }

    #[test]
    fn test_jsx_fragment() {
        let input = r#"const el = <><div>A</div><div>B</div></>;"#;
        let output = transpile_ts(input, "test.tsx").unwrap();
        assert!(output.contains("note"));
    }

    #[test]
    fn test_jsx_with_props() {
        let input = r#"const el = <input type="text" disabled />;"#;
        let output = transpile_ts(input, "test.tsx").unwrap();
        assert!(output.contains(r#"note("input""#));
        assert!(output.contains(r#"type"#));
    }
}
