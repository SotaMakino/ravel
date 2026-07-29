use rquickjs::{AsyncContext, AsyncRuntime};
use std::future::pending;
use std::time::Instant;

use crate::error::{format_thrown, report_uncaught};
use crate::timer::{TimerKind, TimerState, get_timer_state};

/// The event loop.
///
/// One loop, one turn at a time:
///
///   1. Drain the microtask queue: promise callbacks and `await` continuations.
///   2. Fire the timer that has come due, if one has.
///   3. Park until the next timer deadline or until I/O completes, whichever
///      comes first.
///
/// The loop exits when there is nothing left to wait for: no live timers and no
/// I/O in flight. Step 3 never polls. It sleeps on the exact deadline and on
/// wakers registered by the pending reads, so an idle runtime uses no CPU
/// whether the next timer is 1ms or an hour away.
///
/// Timers and I/O wait *together*, which is what makes this one loop rather
/// than two phases: a `setTimeout` still fires on schedule while a read is in
/// flight, and a module sitting on a top-level `await` does not stop either.
pub struct EventLoop<'a> {
    runtime: &'a AsyncRuntime,
    context: &'a AsyncContext,
}

impl<'a> EventLoop<'a> {
    pub fn new(runtime: &'a AsyncRuntime, context: &'a AsyncContext) -> Self {
        Self { runtime, context }
    }

    /// Run until nothing is left to do. Returns false if any job or timer
    /// callback threw an uncaught error.
    pub async fn run(&self) -> bool {
        let mut ok = true;
        loop {
            ok &= self.drain_microtasks().await;

            let timers = get_timer_state();
            if let Some((id, kind)) = timers.as_ref().and_then(|t| t.take_due(Instant::now())) {
                ok &= self.fire_timer(id, kind).await;
                // A callback can queue microtasks and schedule more timers, so
                // start the turn over instead of firing the next one blind.
                continue;
            }

            let deadline = timers.as_ref().and_then(TimerState::next_deadline);
            // The runtime reports both queued jobs and futures spawned by
            // `fs.readFile`; after the drain above, only the latter can be left.
            let io_in_flight = self.runtime.is_job_pending().await;
            if deadline.is_none() && !io_in_flight {
                break;
            }

            self.park(deadline, io_in_flight, timers).await;
        }
        ok
    }

    /// Run queued jobs until the queue is empty, reporting anything that threw.
    ///
    /// This is deliberately not `AsyncRuntime::idle`: idle prints job errors
    /// itself, with `println!`, which would land in the middle of a build's
    /// stdout. Reporting them here keeps them on stderr.
    async fn drain_microtasks(&self) -> bool {
        let mut ok = true;
        loop {
            match self.runtime.execute_pending_job().await {
                Ok(true) => continue,
                Ok(false) => break,
                Err(exception) => {
                    ok = false;
                    let ctx = exception.0.clone();
                    rquickjs::async_with!(ctx => |ctx| {
                        eprintln!("Uncaught {}", format_thrown(&ctx.catch()));
                    })
                    .await;
                }
            }
        }
        ok
    }

    /// Sleep until something needs attention.
    async fn park(
        &self,
        deadline: Option<Instant>,
        io_in_flight: bool,
        timers: Option<TimerState>,
    ) {
        tokio::select! {
            // Woken by the reads themselves: each completion settles its
            // promise and runs the callbacks that were waiting on it.
            _ = drive_io(self.runtime, io_in_flight) => {}
            // Woken by the clock, exactly once, at the deadline.
            _ = sleep_until(deadline) => {}
            // Woken by JavaScript: a callback that ran while we were driving
            // I/O may have scheduled a timer nearer than the deadline above.
            _ = timer_scheduled(timers) => {}
        }
    }

    /// Returns false if the callback threw.
    async fn fire_timer(&self, id: u32, kind: TimerKind) -> bool {
        let ctx = self.context.clone();
        rquickjs::async_with!(ctx => |ctx| {
            let call = match kind {
                TimerKind::Timeout => format!("__ravel_fire_timer({})", id),
                TimerKind::Interval => format!("__ravel_fire_interval({})", id),
            };
            match ctx.eval::<(), _>(call) {
                Ok(()) => true,
                Err(e) => {
                    report_uncaught(&ctx, &e);
                    false
                }
            }
        })
        .await
    }
}

/// Drive spawned I/O to completion, or wait forever when there is none. The
/// select arm needs a future either way; `pending` is the one that never wins.
async fn drive_io(runtime: &AsyncRuntime, in_flight: bool) {
    if in_flight {
        runtime.idle().await
    } else {
        pending::<()>().await
    }
}

async fn sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline.into()).await,
        None => pending::<()>().await,
    }
}

async fn timer_scheduled(timers: Option<TimerState>) {
    match timers {
        Some(timers) => timers.scheduled().await,
        None => pending::<()>().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::set_timer_state;
    use std::time::Duration;

    #[tokio::test]
    async fn test_sleep_until_none_never_completes() {
        let timeout = tokio::time::timeout(Duration::from_millis(20), sleep_until(None)).await;
        assert!(
            timeout.is_err(),
            "a missing deadline must not wake the loop"
        );
    }

    #[tokio::test]
    async fn test_sleep_until_waits_for_the_deadline() {
        let start = Instant::now();
        sleep_until(Some(start + Duration::from_millis(30))).await;
        assert!(start.elapsed() >= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn test_sleep_until_past_deadline_returns_immediately() {
        let start = Instant::now();
        sleep_until(Some(start - Duration::from_secs(1))).await;
        assert!(start.elapsed() < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_timer_scheduled_none_never_completes() {
        let timeout = tokio::time::timeout(Duration::from_millis(20), timer_scheduled(None)).await;
        assert!(timeout.is_err());
    }

    #[tokio::test]
    async fn test_timer_scheduled_wakes_on_a_new_timer() {
        let timers = TimerState::new();
        let waiter = timers.clone();
        let handle = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(2), timer_scheduled(Some(waiter))).await
        });
        tokio::time::sleep(Duration::from_millis(10)).await;
        timers.schedule(1, Duration::from_secs(60), TimerKind::Timeout);
        assert!(
            handle.await.unwrap().is_ok(),
            "scheduling must wake the loop"
        );
    }

    #[tokio::test]
    async fn test_empty_loop_exits_without_hanging() {
        set_timer_state(TimerState::new());
        let runtime = AsyncRuntime::new().unwrap();
        let context = AsyncContext::full(&runtime).await.unwrap();
        let event_loop = EventLoop::new(&runtime, &context);
        let finished = tokio::time::timeout(Duration::from_secs(2), event_loop.run()).await;
        assert!(finished.expect("loop should exit"));
    }

    #[tokio::test]
    async fn test_loop_runs_a_timer_and_then_exits() {
        let timers = TimerState::new();
        set_timer_state(timers.clone());
        let runtime = AsyncRuntime::new().unwrap();
        let context = AsyncContext::full(&runtime).await.unwrap();
        rquickjs::async_with!(context => |ctx| {
            crate::timer::setup_timers(&ctx).unwrap();
            ctx.eval::<(), _>("var fired = false; setTimeout(() => { fired = true; }, 20);")
                .unwrap();
        })
        .await;

        let event_loop = EventLoop::new(&runtime, &context);
        let ok = tokio::time::timeout(Duration::from_secs(2), event_loop.run())
            .await
            .expect("loop should exit once the timer has fired");
        assert!(ok);

        let fired = rquickjs::async_with!(context => |ctx| {
            ctx.eval::<bool, _>("fired").unwrap()
        })
        .await;
        assert!(fired);
        assert!(!timers.has_pending());
    }

    #[tokio::test]
    async fn test_loop_reports_a_throwing_timer_callback() {
        set_timer_state(TimerState::new());
        let runtime = AsyncRuntime::new().unwrap();
        let context = AsyncContext::full(&runtime).await.unwrap();
        rquickjs::async_with!(context => |ctx| {
            crate::timer::setup_timers(&ctx).unwrap();
            ctx.eval::<(), _>("setTimeout(() => { null.x; }, 1);").unwrap();
        })
        .await;

        let event_loop = EventLoop::new(&runtime, &context);
        let ok = tokio::time::timeout(Duration::from_secs(2), event_loop.run())
            .await
            .expect("loop should exit");
        assert!(!ok, "a throwing callback must fail the run");
    }
}
