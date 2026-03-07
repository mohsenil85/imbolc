// Suppress clippy warnings inherited from copied imbolc-ui files
#![allow(
    clippy::new_without_default,
    clippy::should_implement_trait,
    clippy::len_without_is_empty,
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::single_match
)]

pub use imbolc_audio as audio;
pub use imbolc_core::action;
pub use imbolc_core::config;
pub use imbolc_core::dispatch;
pub use imbolc_core::midi;
pub use imbolc_core::state;

mod buffer;
mod chrome;
mod input;
mod runtime;
mod setup;
pub mod ui;
mod window;

use simplelog::{CombinedLogger, LevelFilter, WriteLogger};
use std::fs;
use std::path::PathBuf;

fn init_logging() {
    let log_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("imbolc");
    let _ = fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("imbolc-ui2.log");
    if let Ok(file) = fs::File::create(log_path) {
        let _ = CombinedLogger::init(vec![WriteLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            file,
        )]);
    }
}

fn main() -> std::io::Result<()> {
    init_logging();

    // Install panic hook to restore terminal on crash
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        default_hook(info);
    }));

    let mut backend = ui::RatatuiBackend::new()?;
    backend.start()?;
    let result = runtime::run(&mut backend);
    backend.stop()?;
    result
}
