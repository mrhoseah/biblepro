use serde::{Deserialize, Serialize};

// ── background design types ───────────────────────────────────────────────────

/// Fill mode for a background design layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BgMode {
    Solid,
    LinearH,  // horizontal left → right
    LinearV,  // vertical top → bottom
    Diagonal, // top-left → bottom-right
    Radial,   // center glow outward
    Vignette, // dark edges, centre bright
}

impl Default for BgMode {
    fn default() -> Self {
        Self::Solid
    }
}

/// A single colour stop in a gradient.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BgStop {
    /// Position in [0.0, 1.0].
    pub pos: f32,
    pub color: Rgba,
}

/// Full background design (used for both the frame background and the band fill).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackgroundDesign {
    pub mode: BgMode,
    pub stops: Vec<BgStop>,
}

impl BackgroundDesign {
    pub fn solid(color: Rgba) -> Self {
        Self {
            mode: BgMode::Solid,
            stops: vec![BgStop { pos: 0.0, color }],
        }
    }
    pub fn two_stop(mode: BgMode, c1: Rgba, c2: Rgba) -> Self {
        Self {
            mode,
            stops: vec![
                BgStop {
                    pos: 0.0,
                    color: c1,
                },
                BgStop {
                    pos: 1.0,
                    color: c2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub fn chroma_green() -> Self {
        Self::new(0, 177, 64, 255)
    }
    pub fn black() -> Self {
        Self::new(0, 0, 0, 255)
    }
    pub fn white() -> Self {
        Self::new(255, 255, 255, 255)
    }
    pub fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self::black()
    }
}

/// Available presentation layouts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Template {
    /// Full background fill, text anywhere on frame.
    FullScreen,
    /// Solid band at the bottom; background colour fills above (use chroma green).
    LowerThird,
    /// Lower third band with a coloured accent bar across its top edge.
    LowerThirdAccent,
    /// Lower third band with accent bar AND the reference sitting *on* the bar.
    LowerThirdSplit,
    /// Semi-transparent rounded card centred on the frame.
    CardCenter,
    /// Text only — no background box; use with chroma key or transparent window.
    MinimalText,
}

impl Default for Template {
    fn default() -> Self {
        Self::FullScreen
    }
}

impl Template {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FullScreen => "Full Screen",
            Self::LowerThird => "Lower Third",
            Self::LowerThirdAccent => "Lower Third + Accent",
            Self::LowerThirdSplit => "Lower Third Split",
            Self::CardCenter => "Card Centre",
            Self::MinimalText => "Minimal Text",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::FullScreen => "Solid background, verse anywhere",
            Self::LowerThird => "Band at bottom, transparent above",
            Self::LowerThirdAccent => "Band + top accent strip",
            Self::LowerThirdSplit => "Reference on accent bar, verse in band",
            Self::CardCenter => "Semi-transparent rounded card",
            Self::MinimalText => "Text only, no box",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TextPosition {
    Center,
    LowerThird,
    UpperThird,
}

impl Default for TextPosition {
    fn default() -> Self {
        Self::Center
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentConfig {
    pub ndi_name: String,
    pub width: u32,
    pub height: u32,

    // Layout template
    pub template: Template,

    // Colours
    pub background: Rgba,
    pub band_color: Rgba,   // lower-third band fill
    pub accent_color: Rgba, // accent bar fill
    pub verse_color: Rgba,
    pub reference_color: Rgba,

    // Typography
    pub verse_font_size: f32,
    pub reference_font_size: f32,
    pub line_spacing: f32,

    // Layout (used in FullScreen / CardCenter)
    pub position: TextPosition,
    pub padding_x: f32,

    // Template-specific sizing
    /// Lower third: band height as fraction of frame height (0.0–1.0).
    pub band_height: f32,
    /// Lower third accent bar height in pixels.
    pub accent_px: f32,
    /// Card template: corner radius in pixels.
    pub card_radius: f32,
    /// Card template: background alpha (0–255) for the card fill.
    pub card_alpha: u8,

    pub show_reference: bool,

    /// Canvas-designed background (overrides `background` when Some).
    pub bg_design: Option<BackgroundDesign>,
    /// Canvas-designed band fill for lower-third templates (overrides `band_color` when Some).
    pub band_design: Option<BackgroundDesign>,
}

impl Default for PresentConfig {
    fn default() -> Self {
        Self {
            ndi_name: "BiblePro".into(),
            width: 1920,
            height: 1080,
            template: Template::default(),
            background: Rgba::black(),
            band_color: Rgba::new(15, 15, 30, 230),
            accent_color: Rgba::new(200, 160, 80, 255),
            verse_color: Rgba::white(),
            reference_color: Rgba::new(200, 160, 80, 255),
            verse_font_size: 68.0,
            reference_font_size: 38.0,
            line_spacing: 1.35,
            position: TextPosition::Center,
            padding_x: 0.08,
            band_height: 0.30,
            accent_px: 8.0,
            card_radius: 16.0,
            card_alpha: 210,
            show_reference: true,
            bg_design: None,
            band_design: None,
        }
    }
}
