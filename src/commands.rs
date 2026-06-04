#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// ── arg types ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct NoArgs {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChapterArg<'a> {
    translation_id: &'a str,
    book_id: i32,
    chapter: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchArg<'a> {
    translation_id: &'a str,
    query: &'a str,
    limit: Option<i32>,
    book_id: Option<i32>,
    testament: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RefArg<'a> {
    translation_id: &'a str,
    reference: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchArg<'a> {
    translation_id: &'a str,
    reference: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PickImportArg<'a> {
    translation_id: &'a str,
    translation_name: &'a str,
    language: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PathImportArg<'a> {
    file_path: &'a str,
    translation_id: &'a str,
    translation_name: &'a str,
    language: &'a str,
}

// ── shared data types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Translation {
    pub id: String,
    pub name: String,
    pub abbreviation: String,
    pub language: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Book {
    pub id: i32,
    pub name: String,
    pub short_name: String,
    pub testament: String,
    pub book_order: i32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Verse {
    pub id: i64,
    pub translation_id: String,
    pub book_id: i32,
    pub book_name: String,
    pub chapter: i32,
    pub verse: i32,
    pub text: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SearchResult {
    pub verse: Verse,
    pub snippet: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChapterInfo {
    pub book_name: String,
    pub book_id: i32,
    pub chapter: i32,
    pub total_chapters: i32,
    pub verses: Vec<Verse>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DbStats {
    pub translation_count: i64,
    pub book_count: i64,
    pub verse_count: i64,
    pub translations: Vec<Translation>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImportResult {
    pub translation_id: String,
    pub verses_imported: usize,
    pub message: String,
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn to_js<T: Serialize>(v: &T) -> JsValue {
    serde_wasm_bindgen::to_value(v).unwrap_or(JsValue::NULL)
}

fn from_js<T: for<'de> Deserialize<'de>>(v: JsValue) -> Option<T> {
    serde_wasm_bindgen::from_value(v).ok()
}

// ── command wrappers ──────────────────────────────────────────────────────────

pub async fn cmd_get_books() -> Vec<Book> {
    from_js(invoke("get_books", to_js(&NoArgs {})).await).unwrap_or_default()
}

pub async fn cmd_get_translations() -> Vec<Translation> {
    from_js(invoke("get_translations", to_js(&NoArgs {})).await).unwrap_or_default()
}

pub async fn cmd_get_db_stats() -> Option<DbStats> {
    from_js(invoke("get_db_stats", to_js(&NoArgs {})).await)
}

pub async fn cmd_get_chapter(translation_id: &str, book_id: i32, chapter: i32) -> Option<ChapterInfo> {
    from_js(invoke("get_chapter", to_js(&ChapterArg { translation_id, book_id, chapter })).await)
}

pub async fn cmd_search_verses(
    translation_id: &str,
    query: &str,
    limit: Option<i32>,
    book_id: Option<i32>,
    testament: Option<&str>,
) -> Vec<SearchResult> {
    from_js(invoke(
        "search_verses",
        to_js(&SearchArg { translation_id, query, limit, book_id, testament }),
    ).await)
    .unwrap_or_default()
}

pub async fn cmd_search_by_reference(translation_id: &str, reference: &str) -> Option<Verse> {
    from_js::<Option<Verse>>(invoke(
        "search_by_reference",
        to_js(&RefArg { translation_id, reference }),
    ).await)
    .flatten()
}

pub async fn cmd_fetch_and_cache(translation_id: &str, reference: &str) -> Option<String> {
    invoke("fetch_and_cache_passage", to_js(&FetchArg { translation_id, reference }))
        .await
        .as_string()
}

// ── present / NDI types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Rgba { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

impl Rgba {
    pub fn black() -> Self { Self { r:0, g:0, b:0, a:255 } }
    pub fn white() -> Self { Self { r:255, g:255, b:255, a:255 } }
    pub fn to_hex(&self) -> String { format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b) }
    pub fn to_css(&self) -> String { format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a as f32/255.0) }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BgMode {
    Solid,
    LinearH,
    LinearV,
    Diagonal,
    Radial,
    Vignette,
}

impl BgMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Solid    => "Solid",
            Self::LinearH  => "Linear H",
            Self::LinearV  => "Linear V",
            Self::Diagonal => "Diagonal",
            Self::Radial   => "Radial",
            Self::Vignette => "Vignette",
        }
    }
    pub fn all() -> &'static [BgMode] {
        &[BgMode::Solid, BgMode::LinearH, BgMode::LinearV, BgMode::Diagonal, BgMode::Radial, BgMode::Vignette]
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct BgStop {
    pub pos: f32,
    pub color: Rgba,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct BackgroundDesign {
    pub mode:  BgMode,
    pub stops: Vec<BgStop>,
}

impl BackgroundDesign {
    pub fn solid(color: Rgba) -> Self {
        Self { mode: BgMode::Solid, stops: vec![BgStop { pos: 0.0, color }] }
    }
    pub fn two_stop(mode: BgMode, c1: Rgba, c2: Rgba) -> Self {
        Self { mode, stops: vec![BgStop { pos: 0.0, color: c1 }, BgStop { pos: 1.0, color: c2 }] }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum TextPosition { Center, LowerThird, UpperThird }

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum Template {
    FullScreen,
    LowerThird,
    LowerThirdAccent,
    LowerThirdSplit,
    CardCenter,
    MinimalText,
}

impl Template {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FullScreen        => "Full Screen",
            Self::LowerThird        => "Lower Third",
            Self::LowerThirdAccent  => "Lower Third + Accent",
            Self::LowerThirdSplit   => "Lower Third Split",
            Self::CardCenter        => "Card Centre",
            Self::MinimalText       => "Minimal Text",
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            Self::FullScreen        => "Solid background, verse anywhere",
            Self::LowerThird        => "Band at bottom, transparent above",
            Self::LowerThirdAccent  => "Band + top accent strip",
            Self::LowerThirdSplit   => "Reference on accent bar, verse in band",
            Self::CardCenter        => "Semi-transparent rounded card",
            Self::MinimalText       => "Text only, no box",
        }
    }
    pub fn all() -> &'static [Template] {
        &[
            Template::FullScreen,
            Template::LowerThird,
            Template::LowerThirdAccent,
            Template::LowerThirdSplit,
            Template::CardCenter,
            Template::MinimalText,
        ]
    }
}

impl Default for Template {
    fn default() -> Self { Self::FullScreen }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PresentConfig {
    pub ndi_name:            String,
    pub width:               u32,
    pub height:              u32,
    pub template:            Template,
    pub background:          Rgba,
    pub band_color:          Rgba,
    pub accent_color:        Rgba,
    pub verse_color:         Rgba,
    pub reference_color:     Rgba,
    pub verse_font_size:     f32,
    pub reference_font_size: f32,
    pub line_spacing:        f32,
    pub position:            TextPosition,
    pub padding_x:           f32,
    pub band_height:         f32,
    pub accent_px:           f32,
    pub card_radius:         f32,
    pub card_alpha:          u8,
    pub show_reference:      bool,
    pub bg_design:           Option<BackgroundDesign>,
    pub band_design:         Option<BackgroundDesign>,
    // Media background (image/video)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_image_b64:        Option<String>,
    #[serde(default = "default_bg_opacity")]
    pub bg_image_opacity:    f32,
    #[serde(default = "default_bg_fit")]
    pub bg_image_fit:        String,
    // Typography
    #[serde(default = "default_font_family")]
    pub verse_font_family:   String,   // "serif" | "sans" | "mono"
    #[serde(default)]
    pub verse_bold:          bool,
    #[serde(default)]
    pub verse_italic:        bool,
    #[serde(default = "default_text_align")]
    pub text_align:          String,   // "left" | "center" | "right"
    #[serde(default = "default_text_shadow")]
    pub text_shadow:         bool,
    #[serde(default = "default_letter_spacing")]
    pub letter_spacing:      f32,      // em units, e.g. 0.02
}

fn default_bg_opacity()     -> f32    { 1.0 }
fn default_bg_fit()         -> String { "cover".to_string() }
fn default_font_family()    -> String { "serif".to_string() }
fn default_text_align()     -> String { "center".to_string() }
fn default_text_shadow()    -> bool   { true }
fn default_letter_spacing() -> f32   { 0.01 }

impl Default for PresentConfig {
    fn default() -> Self {
        Self {
            ndi_name:            "BiblePro".into(),
            width:               1920,
            height:              1080,
            template:            Template::default(),
            background:          Rgba { r: 0,   g: 0,   b: 0,   a: 255 },
            band_color:          Rgba { r: 15,  g: 15,  b: 30,  a: 230 },
            accent_color:        Rgba { r: 200, g: 160, b: 80,  a: 255 },
            verse_color:         Rgba { r: 255, g: 255, b: 255, a: 255 },
            reference_color:     Rgba { r: 200, g: 160, b: 80,  a: 255 },
            verse_font_size:     68.0,
            reference_font_size: 38.0,
            line_spacing:        1.35,
            position:            TextPosition::Center,
            padding_x:           0.08,
            band_height:         0.30,
            accent_px:           8.0,
            card_radius:         16.0,
            card_alpha:          210,
            show_reference:      true,
            bg_design:           None,
            band_design:         None,
            bg_image_b64:        None,
            bg_image_opacity:    1.0,
            bg_image_fit:        "cover".to_string(),
            verse_font_family:   "serif".to_string(),
            verse_bold:          false,
            verse_italic:        false,
            text_align:          "center".to_string(),
            text_shadow:         true,
            letter_spacing:      0.01,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PreviewResult {
    pub png_b64: String,
    pub width:   u32,
    pub height:  u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PushVerseArg<'a> { verse_text: &'a str, reference: &'a str }

pub async fn cmd_get_present_config() -> PresentConfig {
    let res = invoke("get_present_config", to_js(&NoArgs {})).await;
    from_js(res).unwrap_or_default()
}

pub async fn cmd_set_present_config(config: &PresentConfig) -> Result<(), String> {
    #[derive(Serialize)]
    struct Arg<'a> { config: &'a PresentConfig }
    let res = invoke("set_present_config", to_js(&Arg { config })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    Ok(())
}

pub async fn cmd_ndi_start() -> Result<String, String> {
    let res = invoke("ndi_start", to_js(&NoArgs {})).await;
    res.as_string().ok_or_else(|| "ndi_start failed".to_string())
}

pub async fn cmd_ndi_stop() -> Result<(), String> {
    invoke("ndi_stop", to_js(&NoArgs {})).await;
    Ok(())
}

pub async fn cmd_ndi_push_verse(verse_text: &str, reference: &str) -> Result<PreviewResult, String> {
    let res = invoke("ndi_push_verse", to_js(&PushVerseArg { verse_text, reference })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "invalid preview response".to_string())
}

pub async fn cmd_ndi_preview(verse_text: &str, reference: &str) -> Result<PreviewResult, String> {
    let res = invoke("ndi_preview", to_js(&PushVerseArg { verse_text, reference })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "invalid preview response".to_string())
}

pub async fn cmd_ndi_clear() -> Result<PreviewResult, String> {
    let res = invoke("ndi_clear", to_js(&NoArgs {})).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "invalid clear response".to_string())
}

pub async fn cmd_ndi_is_active() -> bool {
    let res = invoke("ndi_is_active", to_js(&NoArgs {})).await;
    from_js::<bool>(res).unwrap_or(false)
}

// ── licensing ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Plan { Free, Standard, Premium }

impl Plan {
    pub fn label(&self) -> &'static str {
        match self { Plan::Free => "Free", Plan::Standard => "Standard", Plan::Premium => "Premium" }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LicenseStatus {
    pub plan: Plan,
    pub org: String,
    pub org_id: String,
    pub device_id: String,
    pub expires_at: Option<i64>,
    pub is_in_grace: bool,
    pub grace_days_remaining: Option<i64>,
    pub is_active: bool,
}

#[derive(Serialize)]
struct TokenArg<'a> { token_str: &'a str }

pub async fn cmd_get_license_status() -> Option<LicenseStatus> {
    from_js(invoke("get_license_status", to_js(&NoArgs {})).await)
}

pub async fn cmd_activate_license(token_str: &str) -> Result<LicenseStatus, String> {
    let res = invoke("activate_license", to_js(&TokenArg { token_str })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "Invalid activation response".to_string())
}

pub async fn cmd_deactivate_license() -> Result<(), String> {
    invoke("deactivate_license", to_js(&NoArgs {})).await;
    Ok(())
}

pub async fn cmd_refresh_license() -> Result<LicenseStatus, String> {
    let res = invoke("refresh_license", to_js(&NoArgs {})).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "Invalid refresh response".to_string())
}

// ── output manager types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct MonitorInfo {
    pub index:      usize,
    pub name:       String,
    pub width:      u32,
    pub height:     u32,
    pub x:          i32,
    pub y:          i32,
    pub is_primary: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputKind {
    Ndi     { source_name: String },
    Display { monitor_index: usize, monitor_name: String },
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct OutputInfo {
    pub id:      String,
    pub label:   String,
    pub kind:    OutputKind,
    pub enabled: bool,
    pub active:  bool,
}

// ── output arg types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AddNdiArg<'a> { label: &'a str, source_name: &'a str }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AddDisplayArg<'a> {
    label:         &'a str,
    monitor_index: usize,
    monitor_name:  &'a str,
    x:             i32,
    y:             i32,
    width:         u32,
    height:        u32,
}

#[derive(Serialize)]
struct IdStrArg<'a> { id: &'a str }

// ── output command wrappers ───────────────────────────────────────────────────

pub async fn cmd_list_monitors() -> Result<Vec<MonitorInfo>, String> {
    let res = invoke("list_monitors", to_js(&NoArgs {})).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "invalid monitor list".to_string())
}

pub async fn cmd_get_outputs() -> Vec<OutputInfo> {
    from_js(invoke("get_outputs", to_js(&NoArgs {})).await).unwrap_or_default()
}

pub async fn cmd_add_ndi_output(label: &str, source_name: &str) -> Result<OutputInfo, String> {
    let res = invoke("add_ndi_output", to_js(&AddNdiArg { label, source_name })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "invalid response".to_string())
}

pub async fn cmd_add_display_output(
    label: &str,
    monitor_index: usize,
    monitor_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<OutputInfo, String> {
    let res = invoke(
        "add_display_output",
        to_js(&AddDisplayArg { label, monitor_index, monitor_name, x, y, width, height }),
    ).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "invalid response".to_string())
}

pub async fn cmd_remove_output(id: &str) -> Result<(), String> {
    let res = invoke("remove_output", to_js(&IdStrArg { id })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    Ok(())
}

pub async fn cmd_toggle_output(id: &str) -> Result<OutputInfo, String> {
    let res = invoke("toggle_output", to_js(&IdStrArg { id })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "invalid response".to_string())
}

pub async fn cmd_push_to_all(verse_text: &str, reference: &str) -> Result<PreviewResult, String> {
    let res = invoke("push_to_all", to_js(&PushVerseArg { verse_text, reference })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "invalid preview response".to_string())
}

pub async fn cmd_clear_all() -> Result<PreviewResult, String> {
    let res = invoke("clear_all", to_js(&NoArgs {})).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "invalid clear response".to_string())
}

// ── file importer ─────────────────────────────────────────────────────────────

/// Opens native file picker and imports the selected JSON Bible.
pub async fn cmd_pick_and_import(
    translation_id: &str,
    translation_name: &str,
    language: &str,
) -> Result<ImportResult, String> {
    let res = invoke(
        "pick_and_import",
        to_js(&PickImportArg { translation_id, translation_name, language }),
    )
    .await;

    // Tauri returns errors as rejected promises → JsValue string
    if let Some(err) = res.as_string() {
        return Err(err);
    }
    from_js(res).ok_or_else(|| "Invalid response from pick_and_import".to_string())
}

/// Opens native file picker to select an image; returns base64 data-URL or None if cancelled.
pub async fn cmd_pick_background_image() -> Option<String> {
    from_js::<Option<String>>(invoke("pick_background_image", to_js(&NoArgs {})).await).flatten()
}

/// Imports a Bible translation from a JSON string already downloaded by the frontend.
/// This is the preferred approach — frontend fetches the URL, backend receives the data.
pub async fn cmd_import_from_json(
    json_str: &str,
    translation_id: &str,
    translation_name: &str,
    language: &str,
) -> Result<ImportResult, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Arg<'a> { json_str: &'a str, translation_id: &'a str, translation_name: &'a str, language: &'a str }
    let res = invoke("import_from_json", to_js(&Arg { json_str, translation_id, translation_name, language })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    from_js(res).ok_or_else(|| "Invalid response from import_from_json".to_string())
}

/// Remove an installed translation from the database.
pub async fn cmd_remove_translation(translation_id: &str) -> Result<(), String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Arg<'a> { translation_id: &'a str }
    let res = invoke("remove_translation", to_js(&Arg { translation_id })).await;
    if let Some(e) = res.as_string() { return Err(e); }
    Ok(())
}

// ── study types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Highlight { pub book_id: i32, pub chapter: i32, pub verse: i32, pub color: String }

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Note {
    pub book_id: i32, pub chapter: i32, pub verse: i32,
    pub body: String, pub created_at: i64, pub updated_at: i64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Bookmark {
    pub id: i64, pub book_id: i32, pub chapter: i32, pub verse: i32,
    pub book_name: String, pub verse_text: String, pub label: String, pub created_at: i64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Tag { pub id: i64, pub name: String, pub color: String }

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct StudySet {
    pub id: i64, pub name: String, pub description: String,
    pub verse_count: i64, pub created_at: i64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SetVerse {
    pub book_id: i32, pub chapter: i32, pub verse: i32,
    pub book_name: String, pub verse_text: String, pub note: String, pub sort_order: i32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ReadingPlan {
    pub id: String, pub name: String, pub description: String,
    pub total_days: i32, pub days_done: i32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct PlanDay {
    pub day: i32, pub label: String,
    pub passages: Vec<PlanPassage>, pub completed: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct PlanPassage { pub book_id: i32, pub book_name: String, pub chapter: i32 }

// ── study arg types ───────────────────────────────────────────────────────────

#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct BcvArg { book_id: i32, chapter: i32, verse: i32 }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct BcvColorArg { book_id: i32, chapter: i32, verse: i32, color: String }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct BcvTextArg { book_id: i32, chapter: i32, verse: i32, body: String }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct BcvBookArg { book_id: i32, chapter: i32, verse: i32, book_name: String, verse_text: String }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct BookChArg { book_id: i32, chapter: i32 }
#[derive(Serialize)] struct IdArg { id: i64 }
#[derive(Serialize)] struct SetNameArg<'a> { name: &'a str, description: &'a str }
#[derive(Serialize)] struct TagCreateArg<'a> { name: &'a str, color: &'a str }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct TagVerseArg { tag_id: i64, book_id: i32, chapter: i32, verse: i32 }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct SetVerseArg { set_id: i64, book_id: i32, chapter: i32, verse: i32, book_name: String, verse_text: String }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct SetVerseNoteArg { set_id: i64, book_id: i32, chapter: i32, verse: i32, note: String }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct RemoveSetVerseArg { set_id: i64, book_id: i32, chapter: i32, verse: i32 }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct PlanDayArg<'a> { plan_id: &'a str, day: i32, completed: bool }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] struct PlanIdArg<'a> { plan_id: &'a str }

// ── study command wrappers ────────────────────────────────────────────────────

pub async fn cmd_get_chapter_highlights(book_id: i32, chapter: i32) -> Vec<Highlight> {
    from_js(invoke("get_chapter_highlights", to_js(&BookChArg { book_id, chapter })).await).unwrap_or_default()
}
pub async fn cmd_set_highlight(book_id: i32, chapter: i32, verse: i32, color: String) {
    invoke("set_highlight", to_js(&BcvColorArg { book_id, chapter, verse, color })).await;
}
pub async fn cmd_remove_highlight(book_id: i32, chapter: i32, verse: i32) {
    invoke("remove_highlight", to_js(&BcvArg { book_id, chapter, verse })).await;
}
pub async fn cmd_get_chapter_notes(book_id: i32, chapter: i32) -> Vec<Note> {
    from_js(invoke("get_chapter_notes", to_js(&BookChArg { book_id, chapter })).await).unwrap_or_default()
}
pub async fn cmd_get_all_notes() -> Vec<Note> {
    from_js(invoke("get_all_notes", to_js(&NoArgs {})).await).unwrap_or_default()
}
pub async fn cmd_save_note(book_id: i32, chapter: i32, verse: i32, body: String) {
    invoke("save_note", to_js(&BcvTextArg { book_id, chapter, verse, body })).await;
}
pub async fn cmd_get_bookmarks() -> Vec<Bookmark> {
    from_js(invoke("get_bookmarks", to_js(&NoArgs {})).await).unwrap_or_default()
}
pub async fn cmd_toggle_bookmark(book_id: i32, chapter: i32, verse: i32, book_name: String, verse_text: String) -> bool {
    from_js::<bool>(invoke("toggle_bookmark", to_js(&BcvBookArg { book_id, chapter, verse, book_name, verse_text })).await).unwrap_or(false)
}
pub async fn cmd_delete_bookmark(id: i64) {
    invoke("delete_bookmark", to_js(&IdArg { id })).await;
}
pub async fn cmd_is_bookmarked(book_id: i32, chapter: i32, verse: i32) -> bool {
    from_js::<bool>(invoke("is_bookmarked", to_js(&BcvArg { book_id, chapter, verse })).await).unwrap_or(false)
}
pub async fn cmd_get_study_sets() -> Vec<StudySet> {
    from_js(invoke("get_study_sets", to_js(&NoArgs {})).await).unwrap_or_default()
}
pub async fn cmd_create_study_set(name: &str, description: &str) -> i64 {
    from_js::<i64>(invoke("create_study_set", to_js(&SetNameArg { name, description })).await).unwrap_or(0)
}
pub async fn cmd_delete_study_set(id: i64) {
    invoke("delete_study_set", to_js(&IdArg { id })).await;
}
pub async fn cmd_get_set_verses(set_id: i64) -> Vec<SetVerse> {
    from_js(invoke("get_set_verses", to_js(&IdArg { id: set_id })).await).unwrap_or_default()
}
pub async fn cmd_add_to_study_set(set_id: i64, book_id: i32, chapter: i32, verse: i32, book_name: String, verse_text: String) {
    invoke("add_to_study_set", to_js(&SetVerseArg { set_id, book_id, chapter, verse, book_name, verse_text })).await;
}
pub async fn cmd_remove_from_study_set(set_id: i64, book_id: i32, chapter: i32, verse: i32) {
    invoke("remove_from_study_set", to_js(&RemoveSetVerseArg { set_id, book_id, chapter, verse })).await;
}
pub async fn cmd_update_set_verse_note(set_id: i64, book_id: i32, chapter: i32, verse: i32, note: String) {
    invoke("update_set_verse_note", to_js(&SetVerseNoteArg { set_id, book_id, chapter, verse, note })).await;
}
pub async fn cmd_export_study_set(id: i64) -> Option<String> {
    from_js::<Option<String>>(invoke("export_study_set", to_js(&IdArg { id })).await).flatten()
}
pub async fn cmd_get_reading_plans() -> Vec<ReadingPlan> {
    from_js(invoke("get_reading_plans", to_js(&NoArgs {})).await).unwrap_or_default()
}
pub async fn cmd_get_plan_days(plan_id: &str) -> Vec<PlanDay> {
    from_js(invoke("get_plan_days", to_js(&PlanIdArg { plan_id })).await).unwrap_or_default()
}
pub async fn cmd_mark_plan_day(plan_id: &str, day: i32, completed: bool) {
    invoke("mark_plan_day", to_js(&PlanDayArg { plan_id, day, completed })).await;
}
