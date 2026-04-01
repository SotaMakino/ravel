use javascriptcore::{JSContext, JSException, JSValue};

pub struct PromiseBridge<'a> {
    ctx: &'a JSContext,
}

impl<'a> PromiseBridge<'a> {
    pub fn new(ctx: &'a JSContext) -> Self {
        Self { ctx }
    }

    pub fn setup_promise_helpers(&self) -> Result<(), JSException> {
        let global = self.ctx.global_object()?;

        let rust_future_fn =
            JSValue::new_function(self.ctx, "rustFuture", Some(rust_future_callback));
        global.set_property("rustFuture", rust_future_fn)?;

        Ok(())
    }
}

#[javascriptcore::function_callback]
fn rust_future_callback(
    ctx: &JSContext,
    _function: Option<&javascriptcore::JSObject>,
    _this_object: Option<&javascriptcore::JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Err(JSException::from(JSValue::new_string(
            ctx,
            "rustFuture requires a function",
        )));
    }

    let func = &arguments[0];
    let result = func.as_object()?.call_as_function(None, &[])?;

    if let Ok(s) = result.as_string() {
        println!("rustFuture result: {}", s);
    }

    Ok(result)
}
