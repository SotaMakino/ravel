use javascriptcore::{JSContext, JSException, JSValue};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;
use tokio::task::AbortHandle;
use tokio::time::{sleep, Duration};

static NEXT_TIMER_ID: AtomicU32 = AtomicU32::new(1);

pub fn next_timer_id() -> u32 {
    NEXT_TIMER_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
pub struct TimerEntry {
    pub cancelled: Arc<AtomicBool>,
    pub handle: AbortHandle,
}

#[derive(Debug, Clone)]
pub struct TimerState {
    pub entries: Arc<Mutex<HashMap<u32, TimerEntry>>>,
}

impl TimerState {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, id: u32, cancelled: Arc<AtomicBool>, handle: AbortHandle) {
        self.entries
            .lock()
            .unwrap()
            .insert(id, TimerEntry { cancelled, handle });
    }

    pub fn cancel(&self, id: u32) {
        if let Some(entry) = self.entries.lock().unwrap().get(&id) {
            entry.cancelled.store(true, Ordering::SeqCst);
            entry.handle.abort();
        }
        self.entries.lock().unwrap().remove(&id);
    }

    pub fn has_pending(&self) -> bool {
        !self.entries.lock().unwrap().is_empty()
    }
}

pub struct JscTimerBridge {
    pub state: TimerState,
    pub handle: Handle,
    pub ctx: JSContext,
}

impl JscTimerBridge {
    pub fn new(ctx: JSContext, handle: Handle) -> Self {
        Self {
            state: TimerState::new(),
            handle,
            ctx,
        }
    }

    pub fn setup_timer_globals(&self) -> Result<(), JSException> {
        let global = self.ctx.global_object()?;

        let set_timeout_fn = JSValue::new_function(
            &self.ctx,
            "setTimeout",
            Some(jsc_set_timeout_wrapper),
        );
        global.set_property("setTimeout", set_timeout_fn)?;

        let set_interval_fn = JSValue::new_function(
            &self.ctx,
            "setInterval",
            Some(jsc_set_interval_wrapper),
        );
        global.set_property("setInterval", set_interval_fn)?;

        let clear_timeout_fn = JSValue::new_function(
            &self.ctx,
            "clearTimeout",
            Some(jsc_clear_timeout),
        );
        global.set_property("clearTimeout", clear_timeout_fn)?;

        let clear_interval_fn = JSValue::new_function(
            &self.ctx,
            "clearInterval",
            Some(jsc_clear_interval),
        );
        global.set_property("clearInterval", clear_interval_fn)?;

        Ok(())
    }
}

thread_local! {
    static TIMER_BRIDGE: std::cell::RefCell<Option<Arc<Mutex<JscTimerBridge>>>> = const { std::cell::RefCell::new(None) };
}

pub fn set_timer_bridge(bridge: Arc<Mutex<JscTimerBridge>>) {
    TIMER_BRIDGE.with(|cell| {
        *cell.borrow_mut() = Some(bridge);
    });
}

fn get_timer_bridge() -> Option<Arc<Mutex<JscTimerBridge>>> {
    TIMER_BRIDGE.with(|cell| cell.borrow().clone())
}

#[javascriptcore::function_callback]
fn jsc_set_timeout_wrapper(
    ctx: &JSContext,
    _function: Option<&javascriptcore::JSObject>,
    _this_object: Option<&javascriptcore::JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Err(JSException::from(JSValue::new_string(ctx, "setTimeout requires a callback")));
    }

    let callback = arguments[0].as_string().ok().map(|s| s.to_string()).unwrap_or_default();
    let delay_ms = if arguments.len() >= 2 {
        arguments[1].as_number().unwrap_or(0.0) as u64
    } else {
        0
    };

    let extra_args: Vec<String> = if arguments.len() > 2 {
        arguments[2..]
            .iter()
            .filter_map(|v| v.as_string().ok().map(|s| s.to_string()))
            .collect()
    } else {
        vec![]
    };

    let id = next_timer_id();
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    if let Some(bridge) = get_timer_bridge() {
        let callback_clone = callback.clone();
        let extra_args_clone = extra_args.clone();

        let handle = bridge.lock().unwrap().handle.spawn(async move {
            sleep(Duration::from_millis(delay_ms)).await;
            if cancelled_clone.load(Ordering::SeqCst) {
                return;
            }
            if let Some(b) = get_timer_bridge() {
                b.lock().unwrap().state.entries.lock().unwrap().remove(&id);
            }

            let args_json = extra_args_clone.join(", ");
            let invoke_code = format!("({})({})", callback_clone, args_json);

            if let Some(b) = get_timer_bridge() {
                let ctx_ref = &b.lock().unwrap().ctx;
                let _ = javascriptcore::evaluate_script(ctx_ref, invoke_code.as_str(), None, "timer-callback", 1);
            }
        });

        bridge.lock().unwrap().state.register(id, cancelled, handle.abort_handle());
    }

    Ok(JSValue::new_number(ctx, id as f64))
}

#[javascriptcore::function_callback]
fn jsc_set_interval_wrapper(
    ctx: &JSContext,
    _function: Option<&javascriptcore::JSObject>,
    _this_object: Option<&javascriptcore::JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Err(JSException::from(JSValue::new_string(ctx, "setInterval requires a callback")));
    }

    let callback = arguments[0].as_string().ok().map(|s| s.to_string()).unwrap_or_default();
    let interval_ms = if arguments.len() >= 2 {
        arguments[1].as_number().unwrap_or(0.0) as u64
    } else {
        0
    };

    let extra_args: Vec<String> = if arguments.len() > 2 {
        arguments[2..]
            .iter()
            .filter_map(|v| v.as_string().ok().map(|s| s.to_string()))
            .collect()
    } else {
        vec![]
    };

    let id = next_timer_id();
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    if let Some(bridge) = get_timer_bridge() {
        let callback_clone = callback.clone();
        let extra_args_clone = extra_args.clone();

        let handle = bridge.lock().unwrap().handle.spawn(async move {
            loop {
                sleep(Duration::from_millis(interval_ms)).await;
                if cancelled_clone.load(Ordering::SeqCst) {
                    break;
                }

                let args_json = extra_args_clone.join(", ");
                let invoke_code = format!("({})({})", callback_clone, args_json);

                if let Some(b) = get_timer_bridge() {
                    let ctx_ref = &b.lock().unwrap().ctx;
                    let _ = javascriptcore::evaluate_script(ctx_ref, invoke_code.as_str(), None, "interval-callback", 1);
                }
            }
        });

        bridge.lock().unwrap().state.register(id, cancelled, handle.abort_handle());
    }

    Ok(JSValue::new_number(ctx, id as f64))
}

#[javascriptcore::function_callback]
fn jsc_clear_timeout(
    ctx: &JSContext,
    _function: Option<&javascriptcore::JSObject>,
    _this_object: Option<&javascriptcore::JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_undefined(ctx));
    }

    if let Ok(id) = arguments[0].as_number() {
        if let Some(bridge) = get_timer_bridge() {
            bridge.lock().unwrap().state.cancel(id as u32);
        }
    }

    Ok(JSValue::new_undefined(ctx))
}

#[javascriptcore::function_callback]
fn jsc_clear_interval(
    ctx: &JSContext,
    _function: Option<&javascriptcore::JSObject>,
    _this_object: Option<&javascriptcore::JSObject>,
    arguments: &[JSValue],
) -> Result<JSValue, JSException> {
    if arguments.is_empty() {
        return Ok(JSValue::new_undefined(ctx));
    }

    if let Ok(id) = arguments[0].as_number() {
        if let Some(bridge) = get_timer_bridge() {
            bridge.lock().unwrap().state.cancel(id as u32);
        }
    }

    Ok(JSValue::new_undefined(ctx))
}
