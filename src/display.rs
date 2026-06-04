#![allow(non_snake_case)]

use dioxus::prelude::*;

/// Full-screen slide renderer shown in dedicated monitor output windows.
///
/// Tauri injects PNG slides via `window.eval(...)` — the backend calls:
///   `document.getElementById('bp-slide').src = 'data:image/png;base64,...'`
///
/// The window label starts with "display_" so `main()` routes here.
#[component]
pub fn DisplayApp() -> Element {
    rsx! {
        div { id: "display-root",
            img { id: "bp-slide", src: "", alt: "" }
        }
    }
}
