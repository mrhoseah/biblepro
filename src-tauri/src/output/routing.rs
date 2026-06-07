use serde::{Deserialize, Serialize};

use crate::present::renderer::Frame;

/// Audience role for an output destination.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputRole {
    #[default]
    Program,
    Preview,
    Confidence,
    Stage,
    Lobby,
    Livestream,
}

/// Preferred content source for an output (Auto follows role defaults).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputSource {
    #[default]
    Auto,
    Presentation,
    Scripture,
    Media,
    Countdown,
}

/// How scripture interacts with the base layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScriptureMode {
    /// Scripture replaces the entire frame.
    #[default]
    Replace,
    /// Scripture lower-third overlays the live base layer.
    Overlay,
}

#[derive(Debug, Clone)]
pub struct LayerFrames {
    pub base: Frame,
    pub scripture_full: Option<Frame>,
    pub scripture_overlay: Option<Frame>,
    pub countdown: Option<Frame>,
}

/// Pick the frame to send to a specific output.
pub fn resolve_output_frame(
    role: &OutputRole,
    source: &OutputSource,
    scripture_mode: &ScriptureMode,
    layers: &LayerFrames,
    has_scripture: bool,
    has_countdown: bool,
) -> Frame {
    let effective_source = if *source == OutputSource::Auto {
        default_source_for_role(role)
    } else {
        source.clone()
    };

    match effective_source {
        OutputSource::Scripture => {
            if has_scripture {
                pick_scripture(scripture_mode, layers)
            } else {
                layers.base.clone()
            }
        }
        OutputSource::Countdown => {
            layers
                .countdown
                .clone()
                .unwrap_or_else(|| layers.base.clone())
        }
        OutputSource::Media | OutputSource::Presentation => layers.base.clone(),
        OutputSource::Auto => match role {
            OutputRole::Lobby => {
                if has_countdown {
                    layers.countdown.clone().unwrap_or_else(|| layers.base.clone())
                } else {
                    layers.base.clone()
                }
            }
            OutputRole::Stage => {
                if has_scripture {
                    pick_scripture(scripture_mode, layers)
                } else if has_countdown {
                    layers.countdown.clone().unwrap_or_else(|| layers.base.clone())
                } else {
                    layers.base.clone()
                }
            }
            OutputRole::Confidence => {
                if has_scripture {
                    pick_scripture(&ScriptureMode::Replace, layers)
                } else {
                    layers.base.clone()
                }
            }
            OutputRole::Program | OutputRole::Preview | OutputRole::Livestream => {
                if has_scripture {
                    pick_scripture(scripture_mode, layers)
                } else {
                    layers.base.clone()
                }
            }
        },
    }
}

fn default_source_for_role(role: &OutputRole) -> OutputSource {
    match role {
        OutputRole::Lobby => OutputSource::Countdown,
        OutputRole::Confidence => OutputSource::Scripture,
        _ => OutputSource::Auto,
    }
}

fn pick_scripture(mode: &ScriptureMode, layers: &LayerFrames) -> Frame {
    match mode {
        ScriptureMode::Overlay => layers
            .scripture_overlay
            .clone()
            .or_else(|| layers.scripture_full.clone())
            .unwrap_or_else(|| layers.base.clone()),
        ScriptureMode::Replace => layers
            .scripture_full
            .clone()
            .unwrap_or_else(|| layers.base.clone()),
    }
}
