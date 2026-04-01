use javascriptcore::{JSContext, JSException, JSValue};

pub mod timers;
pub mod promises;
pub mod stdlib;

pub use timers::{JscTimerBridge, TimerState, set_timer_bridge};
pub use promises::PromiseBridge;
pub use stdlib::{setup_math, setup_json, setup_date};

#[javascriptcore::function_callback]
fn console_log(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    let parts: Vec<String> = arguments
        .iter()
        .filter_map(|v| v.as_string().ok().map(|s| s.to_string()))
        .collect();
    println!("{}", parts.join(" "));
    Ok(JSValue::new_undefined(ctx))
}

pub fn setup_console(ctx: &JSContext) -> Result<JSValue, JSException> {
    let console_obj = javascriptcore::evaluate_script(ctx, "({})", None, "console-obj", 1)?;
    let log_fn = JSValue::new_function(ctx, "log", Some(console_log));
    console_obj.as_object()?.set_property("log", log_fn)?;
    Ok(console_obj)
}

pub fn setup_full_environment(ctx: &JSContext) -> Result<(), JSException> {
    let global = ctx.global_object()?;

    let console = setup_console(ctx)?;
    global.set_property("console", console)?;

    let math = setup_math(ctx)?;
    global.set_property("Math", math)?;

    let promise_bridge = PromiseBridge::new(ctx);
    promise_bridge.setup_promise_helpers()?;

    Ok(())
}
