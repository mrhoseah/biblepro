use serde::{Deserialize, Serialize};

use crate::output::routing::{LayerFrames, OutputRole};
use crate::present::config::{PresentConfig, Rgba, Template};
use crate::present::renderer::{render_frame, Frame};

/// Per-output layout profile (beyond source routing).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoleLayout {
    #[default]
    Auto,
    Full,
    StageTimer,
    ConfidenceText,
    LobbyCountdown,
    LivestreamSafe,
}

pub fn default_layout_for_role(role: &OutputRole) -> RoleLayout {
    match role {
        OutputRole::Stage => RoleLayout::StageTimer,
        OutputRole::Confidence => RoleLayout::ConfidenceText,
        OutputRole::Lobby => RoleLayout::LobbyCountdown,
        OutputRole::Livestream => RoleLayout::LivestreamSafe,
        _ => RoleLayout::Full,
    }
}

/// Apply role-specific layout transforms after source routing.
pub fn apply_layout(
    role: &OutputRole,
    layout: &RoleLayout,
    frame: Frame,
    layers: &LayerFrames,
    cfg: &PresentConfig,
) -> Frame {
    let effective = if *layout == RoleLayout::Auto {
        default_layout_for_role(role)
    } else {
        layout.clone()
    };

    match effective {
        RoleLayout::Full | RoleLayout::Auto => frame,
        RoleLayout::StageTimer => render_stage_timer(frame, layers, cfg),
        RoleLayout::ConfidenceText => render_confidence(layers, cfg),
        RoleLayout::LobbyCountdown => layers
            .countdown
            .clone()
            .unwrap_or(frame),
        RoleLayout::LivestreamSafe => render_livestream_safe(frame, cfg),
    }
}

fn render_stage_timer(_base: Frame, layers: &LayerFrames, cfg: &PresentConfig) -> Frame {
    if let Some(cd) = &layers.countdown {
        return cd.clone();
    }
    let mut stage_cfg = cfg.clone();
    stage_cfg.template = Template::FullScreen;
    stage_cfg.background = Rgba::black();
    stage_cfg.verse_color = Rgba::white();
    stage_cfg.verse_font_size = cfg.height as f32 * 0.18;
    stage_cfg.show_reference = false;
    render_frame("STAGE", "", &stage_cfg).unwrap_or_else(|_| layers.base.clone())
}

fn render_confidence(layers: &LayerFrames, cfg: &PresentConfig) -> Frame {
    if let Some(sf) = &layers.scripture_full {
        let mut conf_cfg = cfg.clone();
        conf_cfg.template = Template::FullScreen;
        conf_cfg.background = Rgba::black();
        conf_cfg.verse_font_size = cfg.height as f32 * 0.065;
        conf_cfg.reference_font_size = cfg.height as f32 * 0.035;
        conf_cfg.verse_color = Rgba::white();
        conf_cfg.reference_color = Rgba::new(200, 200, 200, 255);
        conf_cfg.position = crate::present::config::TextPosition::Center;
        // Re-render isn't possible without text — use scripture frame on black base
        let _ = conf_cfg;
        return sf.clone();
    }
    let mut conf_cfg = cfg.clone();
    conf_cfg.background = Rgba::black();
    render_frame("Standby", "", &conf_cfg).unwrap_or_else(|_| layers.base.clone())
}

fn render_livestream_safe(frame: Frame, cfg: &PresentConfig) -> Frame {
    // Letterbox safe zone: scale content to 90% centered
    let margin_x = (cfg.width as f32 * 0.05) as u32;
    let margin_y = (cfg.height as f32 * 0.05) as u32;
    let inner_w = cfg.width - margin_x * 2;
    let inner_h = cfg.height - margin_y * 2;

    let mut out = vec![0u8; (cfg.width * cfg.height * 4) as usize];
    for y in 0..inner_h {
        for x in 0..inner_w {
            let sx = (x as f32 / inner_w as f32 * frame.width as f32) as u32;
            let sy = (y as f32 / inner_h as f32 * frame.height as f32) as u32;
            let si = ((sy.min(frame.height - 1) * frame.width + sx.min(frame.width - 1)) * 4) as usize;
            let di = (((y + margin_y) * cfg.width + (x + margin_x)) * 4) as usize;
            if si + 3 < frame.data.len() && di + 3 < out.len() {
                out[di] = frame.data[si];
                out[di + 1] = frame.data[si + 1];
                out[di + 2] = frame.data[si + 2];
                out[di + 3] = frame.data[si + 3];
            }
        }
    }
    Frame {
        data: out,
        width: cfg.width,
        height: cfg.height,
    }
}
