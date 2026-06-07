use tauri::{AppHandle, Manager, State};

use crate::output::OutputManager;
use crate::output::routing::{OutputRole, OutputSource, ScriptureMode};
use crate::present::PresentState;

use super::engine::ProductionManager;
use super::models::{
    CountdownDef, CountdownSchedule, ProductionPreview, ProductionSnapshot, TransitionTarget,
};
#[tauri::command]
pub fn get_production_state(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
) -> ProductionSnapshot {
    production.snapshot(&outputs)
}

#[tauri::command]
pub fn get_production_preview(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
    present: State<PresentState>,
) -> Result<ProductionPreview, String> {
    let cfg = present.config.lock().unwrap().clone();
    production.preview(&outputs, &cfg)
}

#[tauri::command]
pub fn list_production_countdowns(production: State<ProductionManager>) -> Vec<CountdownDef> {
    production.all_countdowns()
}

#[tauri::command]
pub fn list_production_media(production: State<ProductionManager>) -> Vec<super::models::MediaDef> {
    production.media_store.all_media()
}

#[tauri::command]
pub fn list_production_themes() -> Vec<super::models::ProductionTheme> {
    super::themes::builtin_themes()
}

#[tauri::command]
pub fn set_countdown(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
    id: String,
) -> Result<ProductionSnapshot, String> {
    production.set_countdown(&id, &outputs)
}

#[tauri::command]
pub fn start_countdown(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
) -> Result<ProductionSnapshot, String> {
    production.start_countdown(&outputs)
}

#[tauri::command]
pub fn pause_countdown(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
) -> Result<ProductionSnapshot, String> {
    production.pause_countdown(&outputs)
}

#[tauri::command]
pub fn resume_countdown(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
) -> Result<ProductionSnapshot, String> {
    production.resume_countdown(&outputs)
}

#[tauri::command]
pub fn stop_countdown(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
) -> Result<ProductionSnapshot, String> {
    production.stop_countdown(&outputs)
}

#[tauri::command]
pub fn set_production_media(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
    id: String,
) -> Result<ProductionSnapshot, String> {
    production.set_media(&id, &outputs)
}

#[tauri::command]
pub fn set_media_live(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
    live: bool,
) -> Result<ProductionSnapshot, String> {
    production.set_media_live(live, &outputs)
}

#[tauri::command]
pub fn set_auto_transition(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
    enabled: bool,
    target: TransitionTarget,
) -> Result<ProductionSnapshot, String> {
    production.set_auto_transition(enabled, target, &outputs)
}

#[tauri::command]
pub fn set_scripture_mode(
    outputs: State<OutputManager>,
    mode: ScriptureMode,
) -> Result<(), String> {
    outputs.set_scripture_mode(mode);
    Ok(())
}

#[tauri::command]
pub fn set_output_role(
    outputs: State<OutputManager>,
    id: String,
    role: OutputRole,
) -> Result<crate::output::OutputInfo, String> {
    outputs.set_output_role(&id, role)
}

#[tauri::command]
pub fn set_output_source(
    outputs: State<OutputManager>,
    id: String,
    source: OutputSource,
) -> Result<crate::output::OutputInfo, String> {
    outputs.set_output_source(&id, source)
}

#[tauri::command]
pub fn export_countdown_pack(
    production: State<ProductionManager>,
    id: String,
) -> Result<String, String> {
    production.export_pack(&id)
}

#[tauri::command]
pub fn import_countdown_pack(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
    json: String,
) -> Result<ProductionSnapshot, String> {
    production.import_pack(&json, &outputs)
}

#[tauri::command]
pub fn import_media_file(
    app: AppHandle,
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
    path: String,
    title: String,
    category: String,
) -> Result<ProductionSnapshot, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    production.import_media_file(&app_dir, &path, title, category, &outputs)
}

#[tauri::command]
pub fn import_video_file(
    app: AppHandle,
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
    path: String,
    title: String,
    category: String,
) -> Result<ProductionSnapshot, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    production.import_video_file(&app_dir, &path, title, category, &outputs)
}

#[tauri::command]
pub fn set_countdown_schedule(
    production: State<ProductionManager>,
    outputs: State<OutputManager>,
    schedule: CountdownSchedule,
) -> Result<ProductionSnapshot, String> {
    production.set_countdown_schedule(schedule, &outputs)
}
