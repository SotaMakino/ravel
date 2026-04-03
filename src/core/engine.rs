use rquickjs::{AsyncContext, AsyncRuntime};

pub struct Engine {
    pub runtime: AsyncRuntime,
    pub context: AsyncContext,
}

impl Engine {
    pub async fn new() -> Self {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let context = AsyncContext::full(&runtime)
            .await
            .expect("Failed to create context");
        Self { runtime, context }
    }
}
