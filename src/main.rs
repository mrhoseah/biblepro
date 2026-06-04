mod app;
mod canvas;
mod commands;
mod display;
mod library;
mod license;
mod output;
mod plans;
mod present;
mod reader;
mod search;
mod study;

use app::App;
use display::DisplayApp;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

fn window_label() -> String {
    js_sys::eval(
        "window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label || 'main'"
    )
    .ok()
    .and_then(|v| v.as_string())
    .unwrap_or_else(|| "main".to_string())
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    if window_label().starts_with("display_") {
        launch(DisplayApp);
    } else {
        launch(App);
    }
}
