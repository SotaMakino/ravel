use rquickjs::{Ctx, Result};
use std::path::Path;

pub mod console;
pub mod fs;
pub mod timer;

pub use console::setup_console;
pub use fs::setup_fs;
pub use timer::{TimerMessage, TimerState, get_timer_state, set_timer_state, setup_timers};

pub fn setup_all_apis<'js>(ctx: &Ctx<'js>, root: &Path) -> Result<()> {
    setup_console(ctx)?;
    setup_timers(ctx)?;
    setup_fs(ctx, root)?;
    Ok(())
}
