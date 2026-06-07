use serde::{Deserialize, Serialize};

use crate::present::config::BackgroundDesign;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CountdownStyle {
    Numeric,
    Ring,
    Loader,
    Theme,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CountdownStatus {
    Idle,
    Running,
    Paused,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountdownDef {
    pub id: String,
    pub name: String,
    pub duration: u32,
    pub style: CountdownStyle,
    pub theme_id: String,
    pub headline: String,
    pub subline: String,
    pub loader: String,
    pub media_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountdownRuntime {
    pub def: CountdownDef,
    pub status: CountdownStatus,
    pub remaining_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDef {
    pub id: String,
    pub title: String,
    pub category: String,
    pub media_type: String,
    pub background: BackgroundDesign,
    #[serde(default)]
    pub motion_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountdownSchedule {
    pub enabled: bool,
    pub countdown_id: String,
    pub service_at_unix: u64,
    pub lead_secs: u32,
    pub fired: bool,
}

impl Default for CountdownSchedule {
    fn default() -> Self {
        Self {
            enabled: false,
            countdown_id: "sunday-service".into(),
            service_at_unix: 0,
            lead_secs: 600,
            fired: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaPlaylist {
    pub random_mode: bool,
    pub interval_secs: u32,
}

impl Default for MediaPlaylist {
    fn default() -> Self {
        Self {
            random_mode: false,
            interval_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeAssignment {
    pub content_type: String,
    pub media_id: String,
    pub theme_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSettings {
    pub playlist: MediaPlaylist,
    pub theme_assignments: Vec<ThemeAssignment>,
}

impl Default for MediaSettings {
    fn default() -> Self {
        Self {
            playlist: MediaPlaylist::default(),
            theme_assignments: vec![
                ThemeAssignment {
                    content_type: "scripture".into(),
                    media_id: "gold-clouds".into(),
                    theme_id: "classic-church".into(),
                },
                ThemeAssignment {
                    content_type: "songs".into(),
                    media_id: "blue-motion".into(),
                    theme_id: "worship-glow".into(),
                },
                ThemeAssignment {
                    content_type: "announcements".into(),
                    media_id: "purple-prayer".into(),
                    theme_id: "prayer-soft".into(),
                },
                ThemeAssignment {
                    content_type: "countdowns".into(),
                    media_id: "conference-lines".into(),
                    theme_id: "conference-minimal".into(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountdownRotation {
    pub enabled: bool,
    pub items: Vec<String>,
    pub interval_secs: u32,
}

impl Default for CountdownRotation {
    fn default() -> Self {
        Self {
            enabled: false,
            items: vec![
                "Welcome to Grace Church".into(),
                "Psalm 122:1 — I was glad when they said unto me...".into(),
                "Youth Fellowship Friday".into(),
            ],
            interval_secs: 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionTheme {
    pub id: String,
    pub name: String,
    pub background: BackgroundDesign,
    pub headline_color: crate::present::config::Rgba,
    pub timer_color: crate::present::config::Rgba,
    pub subline_color: crate::present::config::Rgba,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransitionTarget {
    Idle,
    Media,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionSnapshot {
    pub countdown: Option<CountdownRuntime>,
    pub current_media_id: Option<String>,
    pub media_live: bool,
    pub presentation_connected: bool,
    pub active_layer: String,
    pub auto_transition: bool,
    pub transition_target: TransitionTarget,
    pub scripture_mode: String,
    pub custom_countdown_count: usize,
    pub custom_media_count: usize,
    pub schedule: super::scheduler::ScheduleStatus,
    pub rotation: CountdownRotation,
    pub media_settings: MediaSettings,
    pub service_plan: super::plan::ServicePlan,
}

impl Default for ProductionSnapshot {
    fn default() -> Self {
        Self {
            countdown: None,
            current_media_id: None,
            media_live: false,
            presentation_connected: false,
            active_layer: "idle".into(),
            auto_transition: true,
            transition_target: TransitionTarget::Media,
            scripture_mode: "replace".into(),
            custom_countdown_count: 0,
            custom_media_count: 0,
            schedule: super::scheduler::ScheduleStatus {
                schedule: CountdownSchedule::default(),
                seconds_until_start: 0,
                ready: false,
            },
            rotation: CountdownRotation::default(),
            media_settings: MediaSettings::default(),
            service_plan: super::plan::ServicePlan::default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProductionPreview {
    pub png_b64: String,
    pub width: u32,
    pub height: u32,
    pub snapshot: ProductionSnapshot,
}
