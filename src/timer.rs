use rquickjs::{Ctx, Result, Value, function::Rest};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Notify;

static NEXT_TIMER_ID: AtomicU32 = AtomicU32::new(1);

fn next_timer_id() -> u32 {
    NEXT_TIMER_ID.fetch_add(1, Ordering::SeqCst)
}

/// Node clamps sub-millisecond delays to 1ms. Without that, `setInterval(f, 0)`
/// re-arms itself in the past and the loop never gets to park.
const MIN_DELAY: Duration = Duration::from_millis(1);

/// What the loop does with a timer after its callback runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerKind {
    /// Fires once. Already gone by the time the callback runs.
    Timeout,
    /// Re-armed for another round.
    Interval,
}

/// Deadlines in one place, so the loop can ask a single question: when does the
/// next one come due?
///
/// The heap is ordered by `(deadline, id)`, so equal deadlines fire in the order
/// they were scheduled. Cancelling only removes the entry from `live`; the stale
/// heap entry is skipped when it surfaces, which keeps `clearTimeout` O(1).
#[derive(Debug, Default)]
struct TimerQueue {
    heap: BinaryHeap<Reverse<(Instant, u32)>>,
    /// Live timers. The value is the repeat period, or `None` for a timeout.
    live: HashMap<u32, Option<Duration>>,
}

/// Shared handle to the timer queue: JavaScript writes to it through
/// `setTimeout`, and the event loop reads from it.
#[derive(Debug, Clone)]
pub struct TimerState {
    queue: Arc<Mutex<TimerQueue>>,
    /// Raised whenever a timer is scheduled. A loop parked on the old next
    /// deadline needs to know a nearer one just appeared.
    scheduled: Arc<Notify>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerState {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(TimerQueue::default())),
            scheduled: Arc::new(Notify::new()),
        }
    }

    pub fn schedule(&self, id: u32, delay: Duration, kind: TimerKind) {
        let delay = delay.max(MIN_DELAY);
        let period = match kind {
            TimerKind::Timeout => None,
            TimerKind::Interval => Some(delay),
        };
        {
            let mut queue = self.queue.lock().unwrap();
            queue.live.insert(id, period);
            queue.heap.push(Reverse((Instant::now() + delay, id)));
        }
        self.scheduled.notify_one();
    }

    pub fn cancel(&self, id: u32) {
        self.queue.lock().unwrap().live.remove(&id);
    }

    /// When the loop may next have work. `None` means no timers at all.
    pub fn next_deadline(&self) -> Option<Instant> {
        let mut queue = self.queue.lock().unwrap();
        loop {
            let Reverse((deadline, id)) = *queue.heap.peek()?;
            if queue.live.contains_key(&id) {
                return Some(deadline);
            }
            queue.heap.pop();
        }
    }

    /// Take the one timer that is due, if any. Intervals are re-armed from
    /// `now` rather than from the deadline they missed, so a slow callback
    /// cannot leave a burst of catch-up ticks queued behind it.
    pub fn take_due(&self, now: Instant) -> Option<(u32, TimerKind)> {
        let mut queue = self.queue.lock().unwrap();
        loop {
            let Reverse((deadline, id)) = *queue.heap.peek()?;
            if deadline > now {
                return None;
            }
            queue.heap.pop();
            match queue.live.get(&id).copied() {
                None => continue, // cancelled; the entry outlived the timer
                Some(None) => {
                    queue.live.remove(&id);
                    return Some((id, TimerKind::Timeout));
                }
                Some(Some(period)) => {
                    queue.heap.push(Reverse((now + period, id)));
                    return Some((id, TimerKind::Interval));
                }
            }
        }
    }

    pub fn has_pending(&self) -> bool {
        !self.queue.lock().unwrap().live.is_empty()
    }

    /// Completes when a timer is scheduled.
    pub async fn scheduled(&self) {
        self.scheduled.notified().await;
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

fn clear_timer_js(ctx: &Ctx<'_>, id: u32) {
    let _: Result<()> = ctx.eval(format!(
        "if(__ravel_timer_fns[{}]){{delete __ravel_timer_fns[{}];delete __ravel_timer_args[{}];}}",
        id, id, id
    ));
    if let Some(state) = get_timer_state() {
        state.cancel(id);
    }
}

fn schedule_timer<'js>(
    ctx: &Ctx<'js>,
    callback: rquickjs::Function<'js>,
    delay_ms: u64,
    args: Rest<Value<'js>>,
    kind: TimerKind,
) -> Result<u32> {
    let id = next_timer_id();

    let timers_obj = get_or_create_timer_obj(ctx, "__ravel_timer_fns")?;
    timers_obj.set(id, callback)?;

    if !args.0.is_empty() {
        let args_obj = get_or_create_timer_obj(ctx, "__ravel_timer_args")?;
        let args_array = rquickjs::Array::new(ctx.clone())?;
        for (i, arg) in args.0.iter().enumerate() {
            args_array.set(i, arg.clone())?;
        }
        args_obj.set(id, args_array)?;
    }

    if let Some(state) = get_timer_state() {
        state.schedule(id, Duration::from_millis(delay_ms), kind);
    }

    Ok(id)
}

pub fn setup_timers<'js>(ctx: &Ctx<'js>) -> Result<()> {
    ctx.eval::<(), _>(
        r#"
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
    "#,
    )?;

    ctx.globals().set(
        "setTimeout",
        rquickjs::function::Func::new(
            move |ctx: Ctx<'js>,
                  callback: rquickjs::Function<'js>,
                  delay: Option<f64>,
                  args: Rest<Value<'js>>|
                  -> Result<u32> {
                let delay_ms = delay.unwrap_or(0.0).max(0.0) as u64;
                schedule_timer(&ctx, callback, delay_ms, args, TimerKind::Timeout)
            },
        ),
    )?;

    ctx.globals().set(
        "setInterval",
        rquickjs::function::Func::new(
            move |ctx: Ctx<'js>,
                  callback: rquickjs::Function<'js>,
                  interval: Option<f64>,
                  args: Rest<Value<'js>>|
                  -> Result<u32> {
                let interval_ms = interval.unwrap_or(0.0).max(0.0) as u64;
                schedule_timer(&ctx, callback, interval_ms, args, TimerKind::Interval)
            },
        ),
    )?;

    ctx.globals().set(
        "clearTimeout",
        rquickjs::function::Func::new(|ctx: Ctx<'_>, id: u32| {
            clear_timer_js(&ctx, id);
        }),
    )?;

    ctx.globals().set(
        "clearInterval",
        rquickjs::function::Func::new(|ctx: Ctx<'_>, id: u32| {
            clear_timer_js(&ctx, id);
        }),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_state_new_is_empty() {
        let state = TimerState::new();
        assert!(!state.has_pending());
        assert!(state.next_deadline().is_none());
    }

    #[test]
    fn test_schedule_makes_timer_pending() {
        let state = TimerState::new();
        state.schedule(1, Duration::from_millis(50), TimerKind::Timeout);
        assert!(state.has_pending());
        assert!(state.next_deadline().is_some());
    }

    #[test]
    fn test_cancel_removes_timer() {
        let state = TimerState::new();
        state.schedule(1, Duration::from_millis(50), TimerKind::Timeout);
        state.cancel(1);
        assert!(!state.has_pending());
        assert!(state.next_deadline().is_none());
    }

    #[test]
    fn test_cancel_nonexistent_is_a_noop() {
        let state = TimerState::new();
        state.cancel(999);
        assert!(!state.has_pending());
    }

    #[test]
    fn test_take_due_skips_timers_that_are_not_ready() {
        let state = TimerState::new();
        state.schedule(1, Duration::from_secs(60), TimerKind::Timeout);
        assert!(state.take_due(Instant::now()).is_none());
        assert!(state.has_pending());
    }

    #[test]
    fn test_take_due_returns_expired_timeout_once() {
        let state = TimerState::new();
        state.schedule(1, Duration::ZERO, TimerKind::Timeout);
        let later = Instant::now() + Duration::from_millis(10);
        assert_eq!(state.take_due(later), Some((1, TimerKind::Timeout)));
        assert_eq!(state.take_due(later), None);
        assert!(!state.has_pending());
    }

    #[test]
    fn test_take_due_rearms_intervals() {
        let state = TimerState::new();
        state.schedule(1, Duration::from_millis(10), TimerKind::Interval);
        let later = Instant::now() + Duration::from_millis(20);
        assert_eq!(state.take_due(later), Some((1, TimerKind::Interval)));
        // Re-armed, so still pending but not due at the same instant.
        assert!(state.has_pending());
        assert_eq!(state.take_due(later), None);
        assert_eq!(
            state.take_due(later + Duration::from_millis(10)),
            Some((1, TimerKind::Interval))
        );
    }

    #[test]
    fn test_take_due_skips_cancelled_timers() {
        let state = TimerState::new();
        state.schedule(1, Duration::ZERO, TimerKind::Timeout);
        state.schedule(2, Duration::ZERO, TimerKind::Timeout);
        state.cancel(1);
        let later = Instant::now() + Duration::from_millis(10);
        assert_eq!(state.take_due(later), Some((2, TimerKind::Timeout)));
        assert_eq!(state.take_due(later), None);
    }

    #[test]
    fn test_equal_deadlines_fire_in_schedule_order() {
        let state = TimerState::new();
        state.schedule(7, Duration::ZERO, TimerKind::Timeout);
        state.schedule(8, Duration::ZERO, TimerKind::Timeout);
        state.schedule(9, Duration::ZERO, TimerKind::Timeout);
        let later = Instant::now() + Duration::from_millis(10);
        let fired: Vec<u32> = std::iter::from_fn(|| state.take_due(later))
            .map(|(id, _)| id)
            .collect();
        assert_eq!(fired, vec![7, 8, 9]);
    }

    #[test]
    fn test_next_deadline_is_the_earliest_live_timer() {
        let state = TimerState::new();
        state.schedule(1, Duration::from_secs(60), TimerKind::Timeout);
        state.schedule(2, Duration::from_millis(10), TimerKind::Timeout);
        let soon = state.next_deadline().unwrap();
        assert!(soon < Instant::now() + Duration::from_secs(1));
    }

    #[test]
    fn test_next_deadline_ignores_cancelled_head() {
        let state = TimerState::new();
        state.schedule(1, Duration::from_millis(10), TimerKind::Timeout);
        state.schedule(2, Duration::from_secs(60), TimerKind::Timeout);
        state.cancel(1);
        let far = state.next_deadline().unwrap();
        assert!(far > Instant::now() + Duration::from_secs(1));
    }

    #[test]
    fn test_zero_delay_is_clamped_so_the_loop_can_park() {
        let state = TimerState::new();
        state.schedule(1, Duration::ZERO, TimerKind::Timeout);
        assert!(state.next_deadline().unwrap() > Instant::now());
    }

    #[test]
    fn test_next_timer_id_increments() {
        let id1 = next_timer_id();
        let id2 = next_timer_id();
        assert_eq!(id2, id1 + 1);
    }

    #[tokio::test]
    async fn test_scheduling_wakes_a_parked_loop() {
        let state = TimerState::new();
        state.schedule(1, Duration::from_millis(5), TimerKind::Timeout);
        // notify_one holds a permit, so the wait returns even though the
        // schedule happened first.
        state.scheduled().await;
    }

    #[tokio::test]
    async fn test_timer_state_thread_local() {
        set_timer_state(TimerState::new());
        assert!(get_timer_state().is_some());
    }
}
