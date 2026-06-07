use serde::{Deserialize, Serialize};

use super::models::{CountdownDef, ProductionTheme};
use super::themes::{builtin_countdowns, theme_by_id};

#[derive(Debug, Serialize, Deserialize)]
pub struct CountdownPack {
    pub format: String,
    pub version: u32,
    pub countdown: CountdownDef,
    pub theme: Option<ProductionTheme>,
}

impl CountdownPack {
    pub fn new(countdown: CountdownDef, theme: Option<ProductionTheme>) -> Self {
        Self {
            format: "bpcountdown".into(),
            version: 1,
            countdown,
            theme,
        }
    }

    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let pack: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if pack.format != "bpcountdown" {
            return Err("Not a .bpcountdown pack".into());
        }
        Ok(pack)
    }
}

pub fn export_countdown_pack(id: &str) -> Result<String, String> {
    let def = builtin_countdowns()
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("Countdown '{id}' not found"))?;
    let theme = Some(theme_by_id(&def.theme_id));
    CountdownPack::new(def, theme).export_json()
}

pub fn import_countdown_pack(json: &str) -> Result<CountdownDef, String> {
    let pack = CountdownPack::from_json(json)?;
    Ok(pack.countdown)
}
