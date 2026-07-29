use rquickjs::{
    loader::{Loader, Resolver, ScriptLoader},
    AsyncRuntime, Ctx, Error, Module, Promise, Result,
};
use std::path::{Path, PathBuf};

use crate::error::{forget_rejection, report_uncaught};
use crate::transpiler::{is_typescript_file, transpile_ts};

pub async fn setup_module_loader(runtime: &AsyncRuntime, root: &Path) {
    let resolver = ModuleResolver::new(root);
    let loader = TsModuleLoader;
    let js_loader = ScriptLoader::default()
        .with_extension("js")
        .with_extension("mjs")
        .with_extension("ts")
        .with_extension("tsx");
    runtime.set_loader(resolver, (loader, js_loader)).await;
}

struct TsModuleLoader;

impl Loader for TsModuleLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, path: &str) -> Result<Module<'js>> {
        if !is_typescript_file(path) {
            return Err(Error::new_loading(path));
        }

        let source = std::fs::read_to_string(path)
            .map_err(|_| Error::new_loading(path))?;

        let transpiled = transpile_ts(&source, path)
            .map_err(|_| Error::new_loading(path))?;

        Module::declare(ctx.clone(), path, transpiled)
    }
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

/// Where the main module's promise is parked between [`start_module`] and
/// [`finish_module`]. A `Promise<'js>` cannot outlive the context borrow, but a
/// global can.
const MAIN_MODULE_PROMISE: &str = "__ravel_main_module";

/// Start the main module and return as soon as it hits its first `await`.
///
/// Deliberately not awaited here. Awaiting would pin the runtime inside this
/// one call, and the event loop -- which is what makes timers fire and reads
/// complete -- would not get to run until the module was already finished. So
/// evaluation just starts the module; the loop drives the rest.
pub fn start_module<'js>(ctx: &Ctx<'js>, source: &str, filename: &str) -> Result<()> {
    let promise = Module::evaluate(ctx.clone(), filename, source)?;
    ctx.globals().set(MAIN_MODULE_PROMISE, promise)
}

/// Report how the main module ended, once the loop has drained. Returns false
/// if it threw.
///
/// A module still pending here was waiting on something that never settled.
/// There is no work left to make it settle, so let it go quietly.
pub fn finish_module(ctx: &Ctx<'_>) -> bool {
    let Ok(promise) = ctx.globals().get::<_, Promise>(MAIN_MODULE_PROMISE) else {
        return true;
    };
    // The rejection tracker has this promise down as unhandled. Claim it, so
    // the failure is reported once, as an uncaught error rather than a stray
    // rejection.
    forget_rejection(&promise.clone().into_value());
    match promise.result::<()>() {
        Some(Err(e)) => {
            report_uncaught(ctx, &e);
            false
        }
        _ => true,
    }
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
