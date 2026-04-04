use rquickjs::{Ctx, Result, Value, function::Rest};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

static NEXT_TIMER_ID: AtomicU32 = AtomicU32::new(1);

fn next_timer_id() -> u32 {
    NEXT_TIMER_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, Clone)]
pub enum TimerMessage {
    FireTimeout(u32),
    FireInterval(u32),
}

#[derive(Debug)]
pub struct TimerEntry {
    pub cancelled: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct TimerState {
    pub entries: Arc<Mutex<HashMap<u32, TimerEntry>>>,
    pub sender: Arc<mpsc::UnboundedSender<TimerMessage>>,
}

impl TimerState {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<TimerMessage>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                entries: Arc::new(Mutex::new(HashMap::new())),
                sender: Arc::new(tx),
            },
            rx,
        )
    }

    pub fn register(&self, id: u32, cancelled: Arc<AtomicBool>) {
        self.entries
            .lock()
            .unwrap()
            .insert(id, TimerEntry { cancelled });
    }

    pub fn cancel(&self, id: u32) {
        if let Some(entry) = self.entries.lock().unwrap().get(&id) {
            entry.cancelled.store(true, Ordering::SeqCst);
        }
        self.entries.lock().unwrap().remove(&id);
    }

    pub fn has_pending(&self) -> bool {
        !self.entries.lock().unwrap().is_empty()
    }
}

thread_local! {
    static TIMER_STATE: std::cell::RefCell<Option<TimerState>> = const { std::cell::RefCell::new(None) };
}

pub fn set_timer_state(state: TimerState) {
    TIMER_STATE.with(|cell| *cell.borrow_mut() = Some(state));
}

pub fn get_timer_state() -> Option<TimerState> {
    TIMER_STATE.with(|cell| cell.borrow().clone())
}

fn get_or_create_timer_obj<'js>(ctx: &Ctx<'js>, name: &str) -> Result<rquickjs::Object<'js>> {
    let globals = ctx.globals();
    match globals.get(name) {
        Ok(obj) => Ok(obj),
        Err(_) => {
            let o = rquickjs::Object::new(ctx.clone())?;
            globals.set(name, o.clone())?;
            Ok(o)
        }
    }
}

pub fn setup_timers<'js>(ctx: &Ctx<'js>) -> Result<()> {
    ctx.eval::<(), _>(r#"
        var __ravel_timer_fns = {};
        var __ravel_timer_args = {};
        function __ravel_fire_timer(id) {
            var fn = __ravel_timer_fns[id];
            var args = __ravel_timer_args[id] || [];
            delete __ravel_timer_fns[id];
            delete __ravel_timer_args[id];
            if (fn) fn.apply(null, args);
        }
        function __ravel_fire_interval(id) {
            var fn = __ravel_timer_fns[id];
            var args = __ravel_timer_args[id] || [];
            if (fn) fn.apply(null, args);
        }
    "#)?;

    ctx.globals().set(
        "setTimeout",
        rquickjs::function::Func::new(move |ctx: Ctx<'js>, callback: rquickjs::Function<'js>, delay: Option<f64>, args: Rest<Value<'js>>| -> Result<u32> {
            let delay_ms = delay.unwrap_or(0.0).max(0.0) as u64;

            let id = next_timer_id();
            let cancelled = Arc::new(AtomicBool::new(false));
            let cancelled_clone = cancelled.clone();

            let timers_obj = get_or_create_timer_obj(&ctx, "__ravel_timer_fns")?;
            timers_obj.set(id, callback)?;

            if !args.0.is_empty() {
                let args_obj = get_or_create_timer_obj(&ctx, "__ravel_timer_args")?;
                let args_array = rquickjs::Array::new(ctx.clone())?;
                for (i, arg) in args.0.iter().enumerate() {
                    args_array.set(i, arg.clone())?;
                }
                args_obj.set(id, args_array)?;
            }

            if let Some(state) = get_timer_state() {
                state.register(id, cancelled);
                let sender = state.sender.clone();
                tokio::spawn(async move {
                    sleep(Duration::from_millis(delay_ms)).await;
                    if cancelled_clone.load(Ordering::SeqCst) {
                        return;
                    }
                    let _ = sender.send(TimerMessage::FireTimeout(id));
                });
            }

            Ok(id)
        }),
    )?;

    ctx.globals().set(
        "setInterval",
        rquickjs::function::Func::new(move |ctx: Ctx<'js>, callback: rquickjs::Function<'js>, interval: Option<f64>, args: Rest<Value<'js>>| -> Result<u32> {
            let interval_ms = interval.unwrap_or(0.0).max(0.0) as u64;

            let id = next_timer_id();
            let cancelled = Arc::new(AtomicBool::new(false));
            let cancelled_clone = cancelled.clone();

            let timers_obj = get_or_create_timer_obj(&ctx, "__ravel_timer_fns")?;
            timers_obj.set(id, callback)?;

            if !args.0.is_empty() {
                let args_obj = get_or_create_timer_obj(&ctx, "__ravel_timer_args")?;
                let args_array = rquickjs::Array::new(ctx.clone())?;
                for (i, arg) in args.0.iter().enumerate() {
                    args_array.set(i, arg.clone())?;
                }
                args_obj.set(id, args_array)?;
            }

            if let Some(state) = get_timer_state() {
                state.register(id, cancelled);
                let sender = state.sender.clone();
                tokio::spawn(async move {
                    loop {
                        sleep(Duration::from_millis(interval_ms)).await;
                        if cancelled_clone.load(Ordering::SeqCst) {
                            break;
                        }
                        let _ = sender.send(TimerMessage::FireInterval(id));
                    }
                });
            }

            Ok(id)
        }),
    )?;

    ctx.globals().set(
        "clearTimeout",
        rquickjs::function::Func::new(|ctx: Ctx<'_>, id: u32| {
            let _: Result<()> = ctx.eval(format!(
                "if(__ravel_timer_fns[{}]){{delete __ravel_timer_fns[{}];delete __ravel_timer_args[{}];}}",
                id, id, id
            ));
            if let Some(state) = get_timer_state() {
                state.cancel(id);
            }
        }),
    )?;

    ctx.globals().set(
        "clearInterval",
        rquickjs::function::Func::new(|ctx: Ctx<'_>, id: u32| {
            let _: Result<()> = ctx.eval(format!(
                "if(__ravel_timer_fns[{}]){{delete __ravel_timer_fns[{}];delete __ravel_timer_args[{}];}}",
                id, id, id
            ));
            if let Some(state) = get_timer_state() {
                state.cancel(id);
            }
        }),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_state_new() {
        let (state, _rx) = TimerState::new();
        assert!(!state.has_pending());
    }

    #[test]
    fn test_timer_state_register() {
        let (state, _rx) = TimerState::new();
        let cancelled = Arc::new(AtomicBool::new(false));
        state.register(1, cancelled);
        assert!(state.has_pending());
    }

    #[test]
    fn test_timer_state_cancel() {
        let (state, _rx) = TimerState::new();
        let cancelled = Arc::new(AtomicBool::new(false));
        state.register(1, cancelled.clone());
        assert!(state.has_pending());
        state.cancel(1);
        assert!(!state.has_pending());
        assert!(cancelled.load(Ordering::SeqCst));
    }

    #[test]
    fn test_timer_state_multiple() {
        let (state, _rx) = TimerState::new();
        let c1 = Arc::new(AtomicBool::new(false));
        let c2 = Arc::new(AtomicBool::new(false));
        state.register(1, c1);
        state.register(2, c2);
        assert!(state.has_pending());
        state.cancel(1);
        assert!(state.has_pending());
        state.cancel(2);
        assert!(!state.has_pending());
    }

    #[test]
    fn test_timer_state_cancel_nonexistent() {
        let (state, _rx) = TimerState::new();
        state.cancel(999);
        assert!(!state.has_pending());
    }

    #[test]
    fn test_next_timer_id_increments() {
        let id1 = next_timer_id();
        let id2 = next_timer_id();
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_timer_message_variants() {
        let msg1 = TimerMessage::FireTimeout(1);
        let msg2 = TimerMessage::FireInterval(2);
        assert!(matches!(msg1, TimerMessage::FireTimeout(1)));
        assert!(matches!(msg2, TimerMessage::FireInterval(2)));
    }

    #[tokio::test]
    async fn test_timer_state_thread_local() {
        let (state, _rx) = TimerState::new();
        set_timer_state(state.clone());
        let retrieved = get_timer_state();
        assert!(retrieved.is_some());
    }
}
