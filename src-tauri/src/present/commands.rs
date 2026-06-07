use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::Serialize;
use tauri::State;

use super::{
    config::{PresentConfig, Template},
    ndi::{render_and_send, NdiSender},
    renderer::frame_to_png,
    PresentState,
};
use crate::license::{Feature, LicenseState};

#[derive(Serialize)]
pub struct PreviewResult {
    /// Base-64 encoded PNG — displayed directly in the UI as a data URI.
    pub png_b64: String,
    pub width: u32,
    pub height: u32,
}

/// Return the current config.
#[tauri::command]
pub fn get_present_config(state: State<PresentState>) -> PresentConfig {
    state.config.lock().unwrap().clone()
}

/// Save a new config (does not restart NDI).
/// Gates Standard-tier features before applying.
#[tauri::command]
pub fn set_present_config(
    state: State<PresentState>,
    lic: State<LicenseState>,
    config: PresentConfig,
) -> Result<(), String> {
    // ── Standard gates ────────────────────────────────────────────────────────
    let is_advanced_template = matches!(
        config.template,
        Template::LowerThirdAccent | Template::LowerThirdSplit | Template::CardCenter
    );
    if is_advanced_template {
        lic.require_feature(Feature::AdvancedTemplates)?;
    }

    if config.bg_design.is_some() || config.band_design.is_some() {
        lic.require_feature(Feature::CanvasDesign)?;
    }

    if config.width >= 3840 {
        lic.require_feature(Feature::Resolution4K)?;
    }
    // ─────────────────────────────────────────────────────────────────────────

    *state.config.lock().unwrap() = config;
    Ok(())
}

/// Start the NDI sender with the current source name.
#[tauri::command]
pub fn ndi_start(state: State<PresentState>) -> Result<String, String> {
    let name = state.config.lock().unwrap().ndi_name.clone();
    let sender = NdiSender::start(&name)?;
    *state.ndi.0.lock().unwrap() = Some(sender);
    Ok(format!("NDI source '{}' started.", name))
}

/// Stop the NDI sender.
#[tauri::command]
pub fn ndi_stop(state: State<PresentState>) -> Result<(), String> {
    *state.ndi.0.lock().unwrap() = None;
    Ok(())
}

/// Render a verse and:
///   • send it over NDI (if started)
///   • return a PNG preview as base-64
#[tauri::command]
pub fn ndi_push_verse(
    state: State<PresentState>,
    verse_text: String,
    reference: String,
) -> Result<PreviewResult, String> {
    let cfg = state.config.lock().unwrap().clone();
    let frame = render_and_send(&state.ndi, &verse_text, &reference, &cfg)?;

    // Save last pushed verse for the UI
    *state.last_verse.lock().unwrap() = Some((verse_text, reference));

    let png = frame_to_png(&frame)?;
    Ok(PreviewResult {
        png_b64: B64.encode(&png),
        width: frame.width,
        height: frame.height,
    })
}

/// Render a preview frame WITHOUT sending over NDI.
#[tauri::command]
pub fn ndi_preview(
    state: State<PresentState>,
    verse_text: String,
    reference: String,
) -> Result<PreviewResult, String> {
    let cfg = state.config.lock().unwrap().clone();
    let frame = super::renderer::render_frame(&verse_text, &reference, &cfg)?;
    let png = frame_to_png(&frame)?;
    Ok(PreviewResult {
        png_b64: B64.encode(&png),
        width: frame.width,
        height: frame.height,
    })
}

/// Push a blank (background-only) frame — useful to clear the output.
#[tauri::command]
pub fn ndi_clear(state: State<PresentState>) -> Result<PreviewResult, String> {
    ndi_push_verse(state, String::new(), String::new())
}

/// Whether the NDI sender is currently active.
#[tauri::command]
pub fn ndi_is_active(state: State<PresentState>) -> bool {
    state.ndi.0.lock().unwrap().is_some()
}
