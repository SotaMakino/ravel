use rquickjs::{
    loader::{Resolver, ScriptLoader},
    AsyncRuntime, Ctx, Module, Result,
};
use std::path::{Path, PathBuf};

pub async fn setup_module_loader(runtime: &AsyncRuntime, root: &Path) {
    let resolver = ModuleResolver::new(root);
    let loader = ScriptLoader::default()
        .with_extension("js")
        .with_extension("mjs");
    runtime.set_loader(resolver, loader).await;
}

struct ModuleResolver {
    root: PathBuf,
}

impl ModuleResolver {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
}

impl Resolver for ModuleResolver {
    fn resolve<'js>(
        &mut self,
        _ctx: &Ctx<'js>,
        base: &str,
        name: &str,
    ) -> rquickjs::Result<String> {
        let base_path = PathBuf::from(base);
        let resolved = if name.starts_with("./") || name.starts_with("../") {
            if let Some(parent) = base_path.parent() {
                parent.join(name)
            } else {
                self.root.join(name)
            }
        } else {
            self.root.join(name)
        };

        let resolved = if resolved.extension().is_none() {
            let mut resolved = resolved;
            resolved.set_extension("js");
            resolved
        } else {
            resolved
        };

        let resolved = if resolved.is_relative() {
            self.root.join(&resolved)
        } else {
            resolved
        };

        Ok(resolved.to_string_lossy().to_string())
    }
}

pub async fn run_module<'js>(ctx: &Ctx<'js>, source: &str, filename: &str) -> Result<()> {
    let promise = Module::evaluate(ctx.clone(), filename, source)?;
    promise.into_future::<()>().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::setup_console;
    use rquickjs::{loader::FileResolver, AsyncContext, Module};
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    fn setup_test_dir() -> std::io::Result<(tempfile::TempDir, PathBuf)> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().to_path_buf();
        Ok((temp, root))
    }

    #[test]
    fn test_file_resolver_default_patterns() {
        let resolver = FileResolver::default();
        let debug_str = format!("{:?}", resolver);
        assert!(debug_str.contains("FileResolver"));
    }

    #[test]
    fn test_file_resolver_with_path() {
        let resolver = FileResolver::default().with_path("/some/path");
        let debug_str = format!("{:?}", resolver);
        assert!(debug_str.contains("FileResolver"));
    }

    #[test]
    fn test_file_resolver_with_multiple_patterns() {
        let resolver = FileResolver::default()
            .with_pattern("{}.js")
            .with_pattern("{}.mjs")
            .with_pattern("{}.cjs");
        let debug_str = format!("{:?}", resolver);
        assert!(debug_str.contains("FileResolver"));
    }

    #[test]
    fn test_script_loader_default() {
    let loader = ScriptLoader::default()
        .with_extension("js")
        .with_extension("mjs");
        let debug_str = format!("{:?}", loader);
        assert!(debug_str.contains("ScriptLoader"));
    }

    #[test]
    fn test_script_loader_with_extension() {
        let loader = ScriptLoader::default().with_extension("mjs");
        let debug_str = format!("{:?}", loader);
        assert!(debug_str.contains("ScriptLoader"));
    }

    #[tokio::test]
    async fn test_setup_module_loader_async() {
        let (temp, root) = setup_test_dir().unwrap();
        let runtime = AsyncRuntime::new().unwrap();
        setup_module_loader(&runtime, &root).await;
        drop(temp);
    }

    #[tokio::test]
    async fn test_run_module_basic() {
        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        let result = rquickjs::async_with!(ctx => |ctx| {
            setup_console(&ctx).unwrap();
            let source = "console.log('hello');";
            Module::evaluate(ctx.clone(), "test.js", source)
                .and_then(|p| p.finish::<()>())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_module_with_export() {
        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        let result = rquickjs::async_with!(ctx => |ctx| {
            let source = "export const x = 42;";
            Module::evaluate(ctx.clone(), "export.js", source)
                .and_then(|p| p.finish::<()>())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_module_with_default_export() {
        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        let result = rquickjs::async_with!(ctx => |ctx| {
            let source = "export default function() { return 1; }";
            Module::evaluate(ctx.clone(), "default.js", source)
                .and_then(|p| p.finish::<()>())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_module_with_import() {
        let (temp, root) = setup_test_dir().unwrap();

        let lib_path = root.join("lib.js");
        let mut f = fs::File::create(&lib_path).unwrap();
        f.write_all(b"export function add(a, b) { return a + b; }").unwrap();
        drop(f);

        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        setup_module_loader(&runtime, &root).await;

        let main_path = root.join("main.js");
        let result = rquickjs::async_with!(ctx => |ctx| {
            setup_console(&ctx).unwrap();
            let source = r#"import { add } from "./lib.js"; console.log(add(1, 2));"#;
            Module::evaluate(ctx.clone(), main_path.to_str().unwrap(), source)
                .and_then(|p| p.finish::<()>())
        })
        .await;

        assert!(result.is_ok());
        drop(temp);
    }

    #[tokio::test]
    async fn test_run_module_with_named_and_default_import() {
        let (temp, root) = setup_test_dir().unwrap();

        let lib_path = root.join("utils.js");
        let mut f = fs::File::create(&lib_path).unwrap();
        f.write_all(
            br#"
            export const VERSION = "1.0.0";
            export default function greet(name) { return "Hello, " + name; }
            "#,
        )
        .unwrap();
        drop(f);

        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        setup_module_loader(&runtime, &root).await;

        let main_path = root.join("main.js");
        let result = rquickjs::async_with!(ctx => |ctx| {
            setup_console(&ctx).unwrap();
            let source = r#"
                import greet, { VERSION } from "./utils.js";
                console.log(VERSION);
                console.log(greet("Test"));
            "#;
            Module::evaluate(ctx.clone(), main_path.to_str().unwrap(), source)
                .and_then(|p| p.finish::<()>())
        })
        .await;

        assert!(result.is_ok());
        drop(temp);
    }

    #[tokio::test]
    async fn test_run_module_with_async_code() {
        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        let result = rquickjs::async_with!(ctx => |ctx| {
            let source = r#"
                async function test() {
                    return Promise.resolve(42);
                }
                export const result = await test();
            "#;
            Module::evaluate(ctx.clone(), "async.js", source)
                .and_then(|p| p.finish::<()>())
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_module_with_syntax_error() {
        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        let result = rquickjs::async_with!(ctx => |ctx| {
            let source = "export const x = ;";
            Module::evaluate(ctx.clone(), "error.js", source)
                .and_then(|p| p.finish::<()>())
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_module_with_missing_import() {
        let (temp, root) = setup_test_dir().unwrap();

        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        setup_module_loader(&runtime, &root).await;

        let result = rquickjs::async_with!(ctx => |ctx| {
            let source = r#"import { missing } from "./nonexistent.js";"#;
            Module::evaluate(ctx.clone(), "main.js", source)
                .and_then(|p| p.finish::<()>())
        })
        .await;

        assert!(result.is_err());
        drop(temp);
    }

    #[tokio::test]
    async fn test_run_module_transitive_imports() {
        let (temp, root) = setup_test_dir().unwrap();

        let a_path = root.join("a.js");
        let mut f = fs::File::create(&a_path).unwrap();
        f.write_all(br#"import { b } from "./b.js"; export const a = b + 1;"#)
            .unwrap();
        drop(f);

        let b_path = root.join("b.js");
        let mut f = fs::File::create(&b_path).unwrap();
        f.write_all(br#"import { c } from "./c.js"; export const b = c + 1;"#)
            .unwrap();
        drop(f);

        let c_path = root.join("c.js");
        let mut f = fs::File::create(&c_path).unwrap();
        f.write_all(b"export const c = 10;").unwrap();
        drop(f);

        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        setup_module_loader(&runtime, &root).await;

        let result = rquickjs::async_with!(ctx => |ctx| {
            setup_console(&ctx).unwrap();
            let source = r#"import { a } from "./a.js"; console.log(a);"#;
            let res = Module::evaluate(ctx.clone(), "main.js", source)
                .and_then(|p| p.finish::<()>());
            if let Err(ref _e) = res {
                let exc = ctx.catch();
                eprintln!("Transitive imports error: {:?}", exc);
            }
            res
        })
        .await;

        assert!(result.is_ok());
        drop(temp);
    }

    #[tokio::test]
    async fn test_run_module_mjs_extension() {
        let (temp, root) = setup_test_dir().unwrap();

        let lib_path = root.join("helper.mjs");
        let mut f = fs::File::create(&lib_path).unwrap();
        f.write_all(b"export const value = 123;").unwrap();
        drop(f);

        let runtime = AsyncRuntime::new().unwrap();
        let ctx = AsyncContext::full(&runtime).await.unwrap();

        setup_module_loader(&runtime, &root).await;

        let result = rquickjs::async_with!(ctx => |ctx| {
            setup_console(&ctx).unwrap();
            let source = r#"import { value } from "./helper.mjs"; console.log(value);"#;
            let res = Module::evaluate(ctx.clone(), "main.js", source)
                .and_then(|p| p.finish::<()>());
            if let Err(ref _e) = res {
                let exc = ctx.catch();
                eprintln!("MJS extension error: {:?}", exc);
            }
            res
        })
        .await;

        assert!(result.is_ok());
        drop(temp);
    }
}
