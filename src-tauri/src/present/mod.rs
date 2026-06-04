pub mod commands;
pub mod config;
pub mod ndi;
pub mod renderer;

use std::sync::Mutex;
use config::PresentConfig;
use ndi::NdiState;

/// Tauri-managed state for the presentation module.
pub struct PresentState {
    pub config:     Mutex<PresentConfig>,
    pub ndi:        NdiState,
    pub last_verse: Mutex<Option<(String, String)>>,
}

impl PresentState {
    pub fn new() -> Self {
        Self {
            config:     Mutex::new(PresentConfig::default()),
            ndi:        NdiState::new(),
            last_verse: Mutex::new(None),
        }
    }
}
