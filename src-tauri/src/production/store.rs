use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::media_store::StoredMedia;
use super::models::{CountdownDef, CountdownRotation, CountdownSchedule, MediaSettings};
use super::plan::ServicePlan;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductionLibrary {
    pub countdowns: Vec<CountdownDef>,
    pub schedule: CountdownSchedule,
    pub rotation: CountdownRotation,
    pub media: Vec<StoredMedia>,
    pub media_settings: MediaSettings,
    pub service_plan: ServicePlan,
}

pub fn production_dir(app_dir: &Path) -> PathBuf {
    app_dir.join("production")
}

fn countdowns_path(dir: &Path) -> PathBuf {
    dir.join("countdowns.json")
}

fn schedule_path(dir: &Path) -> PathBuf {
    dir.join("schedule.json")
}

fn rotation_path(dir: &Path) -> PathBuf {
    dir.join("rotation.json")
}

fn media_path(dir: &Path) -> PathBuf {
    dir.join("media.json")
}

pub fn load(app_dir: &Path) -> ProductionLibrary {
    let dir = production_dir(app_dir);
    ProductionLibrary {
        countdowns: read_json(&countdowns_path(&dir)).unwrap_or_default(),
        schedule: read_json(&schedule_path(&dir)).unwrap_or_default(),
        rotation: read_json(&rotation_path(&dir)).unwrap_or_default(),
        media: read_json(&media_path(&dir)).unwrap_or_default(),
        media_settings: read_json(&media_settings_path(&dir)).unwrap_or_default(),
        service_plan: read_json(&service_plan_path(&dir)).unwrap_or_default(),
    }
}

fn media_settings_path(dir: &Path) -> PathBuf {
    dir.join("media_settings.json")
}

fn service_plan_path(dir: &Path) -> PathBuf {
    dir.join("service_plan.json")
}

pub fn save_countdowns(app_dir: &Path, countdowns: &[CountdownDef]) -> Result<(), String> {
    let dir = ensure_dir(app_dir)?;
    write_json(&countdowns_path(&dir), countdowns)
}

pub fn save_schedule(app_dir: &Path, schedule: &CountdownSchedule) -> Result<(), String> {
    let dir = ensure_dir(app_dir)?;
    write_json(&schedule_path(&dir), schedule)
}

pub fn save_rotation(app_dir: &Path, rotation: &CountdownRotation) -> Result<(), String> {
    let dir = ensure_dir(app_dir)?;
    write_json(&rotation_path(&dir), rotation)
}

pub fn save_media(app_dir: &Path, media: &[StoredMedia]) -> Result<(), String> {
    let dir = ensure_dir(app_dir)?;
    write_json(&media_path(&dir), media)
}

pub fn save_media_settings(app_dir: &Path, settings: &MediaSettings) -> Result<(), String> {
    let dir = ensure_dir(app_dir)?;
    write_json(&media_settings_path(&dir), settings)
}

pub fn save_service_plan(app_dir: &Path, plan: &ServicePlan) -> Result<(), String> {
    let dir = ensure_dir(app_dir)?;
    write_json(&service_plan_path(&dir), plan)
}

fn ensure_dir(app_dir: &Path) -> Result<PathBuf, String> {
    let dir = production_dir(app_dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T> {
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}
