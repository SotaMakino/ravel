use rquickjs::AsyncContext;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::timer::{TimerMessage, get_timer_state};

pub struct EventLoop {
    timer_rx: mpsc::UnboundedReceiver<TimerMessage>,
}

impl EventLoop {
    pub fn new(timer_rx: mpsc::UnboundedReceiver<TimerMessage>) -> Self {
        Self { timer_rx }
    }

    pub async fn run(&mut self, ctx: &AsyncContext) {
        loop {
            tokio::select! {
                Some(msg) = self.timer_rx.recv() => {
                    let ctx_clone = ctx.clone();
                    rquickjs::async_with!(ctx_clone => |ctx| {
                        match msg {
                            TimerMessage::FireTimeout(id) => {
                                let _: Result<(), _> = ctx.eval(format!("__ravel_fire_timer({})", id));
                                if let Some(state) = get_timer_state() {
                                    state.entries.lock().unwrap().remove(&id);
                                }
                            }
                            TimerMessage::FireInterval(id) => {
                                let _: Result<(), _> = ctx.eval(format!("__ravel_fire_interval({})", id));
                            }
                        }
                    })
                    .await;
                }
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    if let Some(state) = get_timer_state() {
                        if !state.has_pending() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
