use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::task::AbortHandle;

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
