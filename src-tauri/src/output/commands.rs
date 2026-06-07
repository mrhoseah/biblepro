use tauri::{AppHandle, Manager, State};

use super::ndi_recv::NdiSourceInfo;
use super::routing::OutputRole;
use super::{MonitorInfo, OutputInfo, OutputManager, PushResult, RoleLayout};
use crate::present::PresentState;
use crate::production::ProductionManager;

// ── Monitor discovery ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let main_win = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    let monitors = main_win.available_monitors().map_err(|e| e.to_string())?;

    let primary_pos = main_win
        .primary_monitor()
        .ok()
        .flatten()
        .map(|p| (p.position().x, p.position().y));

    let result = monitors
        .into_iter()
        .enumerate()
        .map(|(i, m)| {
            let pos = (m.position().x, m.position().y);
            MonitorInfo {
                index: i,
                name: m.name().map_or("Monitor", |v| v).to_string(),
                width: m.size().width,
                height: m.size().height,
                x: pos.0,
                y: pos.1,
                is_primary: primary_pos == Some(pos),
            }
        })
        .collect();

    Ok(result)
}

// ── Output CRUD ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_outputs(mgr: State<OutputManager>) -> Vec<OutputInfo> {
    mgr.get_outputs()
}

#[tauri::command]
pub fn add_ndi_output(
    mgr: State<OutputManager>,
    label: String,
    source_name: String,
    role: Option<OutputRole>,
) -> Result<OutputInfo, String> {
    mgr.add_ndi(label, source_name, role.unwrap_or_default())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn add_display_output(
    app: AppHandle,
    mgr: State<OutputManager>,
    label: String,
    monitor_index: usize,
    monitor_name: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    role: Option<OutputRole>,
) -> Result<OutputInfo, String> {
    mgr.add_display(
        &app,
        label,
        monitor_index,
        monitor_name,
        x,
        y,
        width,
        height,
        role.unwrap_or_default(),
    )
}

#[tauri::command]
pub fn remove_output(app: AppHandle, mgr: State<OutputManager>, id: String) -> Result<(), String> {
    mgr.remove(&app, &id)
}

#[tauri::command]
pub fn toggle_output(mgr: State<OutputManager>, id: String) -> Result<OutputInfo, String> {
    mgr.toggle(&id)
}

#[tauri::command]
pub fn set_output_layout(
    mgr: State<OutputManager>,
    id: String,
    layout: RoleLayout,
) -> Result<OutputInfo, String> {
    mgr.set_output_layout(&id, layout)
}

// ── Presentation source ───────────────────────────────────────────────────────

#[tauri::command]
pub fn list_ndi_sources() -> Vec<NdiSourceInfo> {
    OutputManager::list_ndi_sources()
}

#[tauri::command]
pub fn connect_presentation_source(
    app: AppHandle,
    mgr: State<OutputManager>,
    source_name: String,
) -> Result<(), String> {
    mgr.connect_presentation_source(app, source_name)
}

#[tauri::command]
pub fn disconnect_presentation_source(mgr: State<OutputManager>) {
    mgr.disconnect_presentation_source();
}

#[tauri::command]
pub fn get_presentation_preview(mgr: State<OutputManager>) -> Option<String> {
    mgr.get_presentation_preview()
}

// ── Push ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn push_to_all(
    app: AppHandle,
    mgr: State<OutputManager>,
    present: State<PresentState>,
    production: State<ProductionManager>,
    verse_text: String,
    reference: String,
) -> Result<PushResult, String> {
    let cfg = present.config.lock().unwrap().clone();
    *present.last_verse.lock().unwrap() = Some((verse_text.clone(), reference.clone()));
    let base = production.compose_base_for_push(&mgr, &cfg).ok();
    mgr.push_verse(
        &app,
        &verse_text,
        &reference,
        &cfg,
        base.as_ref(),
    )
}

#[tauri::command]
pub fn clear_all(
    app: AppHandle,
    mgr: State<OutputManager>,
    present: State<PresentState>,
) -> Result<PushResult, String> {
    let cfg = present.config.lock().unwrap().clone();
    mgr.clear(&app, &cfg)
}
