use rquickjs::{
    loader::{FileResolver, ScriptLoader},
    AsyncRuntime, Ctx, Module, Result,
};
use std::path::Path;

pub async fn setup_module_loader(runtime: &AsyncRuntime, root: &Path) {
    let root_str = root.to_string_lossy().to_string();

    let resolver = FileResolver::default()
        .with_path(&root_str)
        .with_pattern("{}.js")
        .with_pattern("{}.mjs");

    let loader = ScriptLoader::default();

    runtime.set_loader(resolver, loader).await;
}

pub async fn run_module<'js>(ctx: &Ctx<'js>, source: &str, filename: &str) -> Result<()> {
    let promise = Module::evaluate(ctx.clone(), filename, source)?;
    promise.into_future::<()>().await
}
