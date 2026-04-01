use javascriptcore::{JSContext, JSException, JSObject, JSValue};
use std::time::{SystemTime, UNIX_EPOCH};

#[javascriptcore::function_callback]
fn math_random(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    _arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    let val = fastrand::f64();
    Ok(JSValue::new_number(ctx, val))
}

#[javascriptcore::function_callback]
fn math_floor(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_number(ctx, f64::NAN));
    }
    let num = arguments[0].as_number()?;
    Ok(JSValue::new_number(ctx, num.floor()))
}

#[javascriptcore::function_callback]
fn math_ceil(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_number(ctx, f64::NAN));
    }
    let num = arguments[0].as_number()?;
    Ok(JSValue::new_number(ctx, num.ceil()))
}

#[javascriptcore::function_callback]
fn math_abs(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_number(ctx, f64::NAN));
    }
    let num = arguments[0].as_number()?;
    Ok(JSValue::new_number(ctx, num.abs()))
}

#[javascriptcore::function_callback]
fn math_max(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_number(ctx, f64::NEG_INFINITY));
    }
    let mut max = f64::NEG_INFINITY;
    for arg in arguments {
        let num = arg.as_number()?;
        if num > max {
            max = num;
        }
    }
    Ok(JSValue::new_number(ctx, max))
}

#[javascriptcore::function_callback]
fn math_min(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_number(ctx, f64::INFINITY));
    }
    let mut min = f64::INFINITY;
    for arg in arguments {
        let num = arg.as_number()?;
        if num < min {
            min = num;
        }
    }
    Ok(JSValue::new_number(ctx, min))
}

#[javascriptcore::function_callback]
fn math_pow(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.len() < 2 {
        return Ok(JSValue::new_number(ctx, f64::NAN));
    }
    let base = arguments[0].as_number()?;
    let exp = arguments[1].as_number()?;
    Ok(JSValue::new_number(ctx, base.powf(exp)))
}

#[javascriptcore::function_callback]
fn math_sqrt(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_number(ctx, f64::NAN));
    }
    let num = arguments[0].as_number()?;
    Ok(JSValue::new_number(ctx, num.sqrt()))
}

pub fn setup_math(ctx: &JSContext) -> Result<JSValue, JSException> {
    let math_obj = javascriptcore::evaluate_script(ctx, "({})", None, "math-obj", 1)?;

    let constants = vec![
        ("PI", std::f64::consts::PI),
        ("E", std::f64::consts::E),
        ("SQRT2", std::f64::consts::SQRT_2),
        ("LN2", std::f64::consts::LN_2),
        ("LN10", std::f64::consts::LN_10),
        ("LOG2E", std::f64::consts::LOG2_E),
        ("LOG10E", std::f64::consts::LOG10_E),
    ];

    for (name, value) in constants {
        math_obj
            .as_object()?
            .set_property(name, JSValue::new_number(ctx, value))?;
    }

    type MathFn = unsafe extern "C" fn(
        *const javascriptcore::sys::OpaqueJSContext,
        *mut javascriptcore::sys::OpaqueJSValue,
        *mut javascriptcore::sys::OpaqueJSValue,
        usize,
        *const *const javascriptcore::sys::OpaqueJSValue,
        *mut *const javascriptcore::sys::OpaqueJSValue,
    ) -> *const javascriptcore::sys::OpaqueJSValue;

    let methods: Vec<(&str, MathFn)> = vec![
        ("random", math_random as MathFn),
        ("floor", math_floor as MathFn),
        ("ceil", math_ceil as MathFn),
        ("abs", math_abs as MathFn),
        ("max", math_max as MathFn),
        ("min", math_min as MathFn),
        ("pow", math_pow as MathFn),
        ("sqrt", math_sqrt as MathFn),
    ];

    for (name, callback) in methods {
        let fn_val = JSValue::new_function(ctx, name, Some(callback));
        math_obj.as_object()?.set_property(name, fn_val)?;
    }

    Ok(math_obj)
}

#[javascriptcore::function_callback]
fn json_stringify(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_undefined(ctx));
    }
    let json_str = arguments[0].to_json_string(0)?;
    Ok(JSValue::new_string(ctx, json_str.to_string()))
}

#[javascriptcore::function_callback]
fn json_parse(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_undefined(ctx));
    }
    let json_str = arguments[0].as_string()?;
    let json_str_rust = json_str.to_string();
    let json_code = format!(
        "JSON.parse({})",
        serde_json::to_string(&json_str_rust)
            .map_err(|e| JSException::from(JSValue::new_string(ctx, e.to_string())))?
    );
    javascriptcore::evaluate_script(ctx, json_code.as_str(), None, "json-parse", 1)
}

pub fn setup_json(ctx: &JSContext) -> Result<JSValue, JSException> {
    let json_obj = javascriptcore::evaluate_script(ctx, "({})", None, "json-obj", 1)?;

    let stringify_fn = JSValue::new_function(ctx, "stringify", Some(json_stringify));
    json_obj
        .as_object()?
        .set_property("stringify", stringify_fn)?;

    let parse_fn = JSValue::new_function(ctx, "parse", Some(json_parse));
    json_obj.as_object()?.set_property("parse", parse_fn)?;

    Ok(json_obj)
}

fn do_date_now(ctx: &JSContext) -> Result<JSValue, JSException> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64;
    Ok(JSValue::new_number(ctx, now))
}

#[javascriptcore::function_callback]
fn date_now(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    _arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    do_date_now(ctx)
}

#[javascriptcore::function_callback]
fn date_get_timestamp(
    ctx: &JSContext,
    _function: Option<&JSObject>,
    _this_object: Option<&JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return do_date_now(ctx);
    }
    let date_val = &arguments[0];
    if let Ok(s) = date_val.as_string() {
        let parse_code = format!("new Date({}).getTime()", s);
        javascriptcore::evaluate_script(ctx, parse_code.as_str(), None, "date-parse", 1)
    } else {
        do_date_now(ctx)
    }
}

pub fn setup_date(ctx: &JSContext) -> Result<JSValue, JSException> {
    let date_obj = javascriptcore::evaluate_script(ctx, "({})", None, "date-obj", 1)?;

    let now_fn = JSValue::new_function(ctx, "now", Some(date_now));
    date_obj.as_object()?.set_property("now", now_fn)?;

    let get_timestamp_fn = JSValue::new_function(ctx, "getTimestamp", Some(date_get_timestamp));
    date_obj
        .as_object()?
        .set_property("getTimestamp", get_timestamp_fn)?;

    Ok(date_obj)
}
