pub mod commands;
pub mod ndi_recv;
pub mod role_layouts;
pub mod routing;

pub use role_layouts::RoleLayout;

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::present::config::PresentConfig;
use crate::present::ndi::NdiSender;
use crate::present::renderer::{frame_to_png, render_frame, render_scripture_overlay, Frame};
use routing::{LayerFrames, OutputRole, OutputSource, ScriptureMode};
use ndi_recv::{NdiReceiver, ReceivedFrame};

// ── Monitor info ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

// ── Output types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputKind {
    Ndi {
        source_name: String,
    },
    Display {
        monitor_index: usize,
        monitor_name: String,
    },
}

/// Serialisable status sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputInfo {
    pub id: String,
    pub label: String,
    pub kind: OutputKind,
    pub enabled: bool,
    pub active: bool,
    pub role: OutputRole,
    pub source: OutputSource,
    #[serde(default)]
    pub layout: RoleLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingSnapshot {
    pub scripture_mode: ScriptureMode,
    pub live_verse_active: bool,
}

/// Push result mirrors PresentResult — includes PNG preview.
#[derive(Debug, Serialize)]
pub struct PushResult {
    pub png_b64: String,
    pub width: u32,
    pub height: u32,
}

// ── Internal entry (not serialised) ──────────────────────────────────────────

struct OutputEntry {
    info: OutputInfo,
    ndi_sender: Option<NdiSender>,
    window_label: Option<String>,
}

// ── Presentation source ───────────────────────────────────────────────────────

struct PresentationSource {
    receiver: NdiReceiver,
}

// ── OutputManager ─────────────────────────────────────────────────────────────

pub struct OutputManager {
    outputs: Arc<Mutex<Vec<OutputEntry>>>,
    presentation: Mutex<Option<PresentationSource>>,
    pub verse_active: Arc<AtomicBool>,
    pub scripture_mode: Mutex<ScriptureMode>,
    pub live_verse: Mutex<Option<(String, String)>>,
}

impl OutputManager {
    pub fn new() -> Self {
        Self {
            outputs: Arc::new(Mutex::new(Vec::new())),
            presentation: Mutex::new(None),
            verse_active: Arc::new(AtomicBool::new(false)),
            scripture_mode: Mutex::new(ScriptureMode::Replace),
            live_verse: Mutex::new(None),
        }
    }

    pub fn routing_snapshot(&self) -> RoutingSnapshot {
        RoutingSnapshot {
            scripture_mode: self.scripture_mode.lock().unwrap().clone(),
            live_verse_active: self.live_verse.lock().unwrap().is_some(),
        }
    }

    pub fn set_scripture_mode(&self, mode: ScriptureMode) {
        *self.scripture_mode.lock().unwrap() = mode;
    }

    pub fn set_output_role(&self, id: &str, role: OutputRole) -> Result<OutputInfo, String> {
        let mut outputs = self.outputs.lock().unwrap();
        let entry = outputs
            .iter_mut()
            .find(|e| e.info.id == id)
            .ok_or_else(|| format!("Output '{id}' not found"))?;
        entry.info.role = role;
        Ok(entry.info.clone())
    }

    pub fn set_output_source(&self, id: &str, source: OutputSource) -> Result<OutputInfo, String> {
        let mut outputs = self.outputs.lock().unwrap();
        let entry = outputs
            .iter_mut()
            .find(|e| e.info.id == id)
            .ok_or_else(|| format!("Output '{id}' not found"))?;
        entry.info.source = source;
        Ok(entry.info.clone())
    }

    pub fn set_output_layout(&self, id: &str, layout: RoleLayout) -> Result<OutputInfo, String> {
        let mut outputs = self.outputs.lock().unwrap();
        let entry = outputs
            .iter_mut()
            .find(|e| e.info.id == id)
            .ok_or_else(|| format!("Output '{id}' not found"))?;
        entry.info.layout = layout;
        Ok(entry.info.clone())
    }

    // ── read ──────────────────────────────────────────────────────────────────

    pub fn get_outputs(&self) -> Vec<OutputInfo> {
        self.outputs
            .lock()
            .unwrap()
            .iter()
            .map(|e| e.info.clone())
            .collect()
    }

    // ── NDI output ────────────────────────────────────────────────────────────

    pub fn add_ndi(
        &self,
        label: String,
        source_name: String,
        role: OutputRole,
    ) -> Result<OutputInfo, String> {
        let sender = NdiSender::start(&source_name)?;
        let id = uuid::Uuid::new_v4().to_string();
        let info = OutputInfo {
            id,
            label,
            kind: OutputKind::Ndi { source_name },
            enabled: true,
            active: true,
            role,
            source: OutputSource::Auto,
            layout: RoleLayout::Auto,
        };
        self.outputs.lock().unwrap().push(OutputEntry {
            info: info.clone(),
            ndi_sender: Some(sender),
            window_label: None,
        });
        Ok(info)
    }

    // ── Display output ────────────────────────────────────────────────────────

    pub fn add_display(
        &self,
        app: &AppHandle,
        label: String,
        monitor_index: usize,
        monitor_name: String,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        role: OutputRole,
    ) -> Result<OutputInfo, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let win_label = format!("display_{}", id.replace('-', ""));

        WebviewWindowBuilder::new(
            app,
            &win_label,
            WebviewUrl::App("index.html?window=display".into()),
        )
        .title(format!("BiblePro — {}", label))
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .position(x as f64, y as f64)
        .inner_size(width as f64, height as f64)
        .build()
        .map_err(|e| e.to_string())?;

        let info = OutputInfo {
            id,
            label,
            kind: OutputKind::Display {
                monitor_index,
                monitor_name,
            },
            enabled: true,
            active: true,
            role,
            source: OutputSource::Auto,
            layout: RoleLayout::Auto,
        };
        self.outputs.lock().unwrap().push(OutputEntry {
            info: info.clone(),
            ndi_sender: None,
            window_label: Some(win_label),
        });
        Ok(info)
    }

    // ── Lifecycle ──────────────────────────────────────────────────────────────

    pub fn remove(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        let mut outputs = self.outputs.lock().unwrap();
        let idx = outputs
            .iter()
            .position(|e| e.info.id == id)
            .ok_or_else(|| format!("Output '{}' not found", id))?;
        let entry = outputs.remove(idx);
        if let Some(win_label) = &entry.window_label {
            if let Some(win) = app.get_webview_window(win_label) {
                let _ = win.close();
            }
        }
        Ok(())
    }

    pub fn toggle(&self, id: &str) -> Result<OutputInfo, String> {
        let mut outputs = self.outputs.lock().unwrap();
        let entry = outputs
            .iter_mut()
            .find(|e| e.info.id == id)
            .ok_or_else(|| format!("Output '{}' not found", id))?;
        entry.info.enabled = !entry.info.enabled;
        Ok(entry.info.clone())
    }

    // ── Presentation source ───────────────────────────────────────────────────

    /// Scan the LAN for NDI sources. Blocks ~2 s.
    pub fn list_ndi_sources() -> Vec<ndi_recv::NdiSourceInfo> {
        ndi_recv::list_ndi_sources()
    }

    /// Connect an NDI source as the presentation background.
    /// Frames are composited by the global production compositor thread.
    pub fn connect_presentation_source(
        &self,
        _app: AppHandle,
        source_name: String,
    ) -> Result<(), String> {
        *self.presentation.lock().unwrap() = None;
        let receiver = ndi_recv::connect_ndi_source(&source_name)?;
        *self.presentation.lock().unwrap() = Some(PresentationSource { receiver });
        Ok(())
    }

    pub fn has_presentation_source(&self) -> bool {
        self.presentation.lock().unwrap().is_some()
    }

    pub fn latest_presentation_frame(&self) -> Option<ReceivedFrame> {
        let guard = self.presentation.lock().unwrap();
        guard.as_ref()?.receiver.latest_frame()
    }

    /// Push routed frames to all enabled outputs.
    pub fn dispatch_routed(&self, app: &AppHandle, layers: &LayerFrames, cfg: &PresentConfig) {
        let scripture_mode = self.scripture_mode.lock().unwrap().clone();
        let has_scripture = layers.scripture_full.is_some();
        let has_countdown = layers.countdown.is_some();
        let outputs = self.outputs.lock().unwrap();

        for entry in outputs.iter() {
            if !entry.info.enabled {
                continue;
            }
            let frame = routing::resolve_output_frame(
                &entry.info.role,
                &entry.info.source,
                &scripture_mode,
                layers,
                has_scripture,
                has_countdown,
            );
            let frame = role_layouts::apply_layout(
                &entry.info.role,
                &entry.info.layout,
                frame,
                layers,
                cfg,
            );
            let uri = frame_to_png(&frame)
                .ok()
                .map(|png| format!("data:image/png;base64,{}", B64.encode(&png)));
            let uri = match uri {
                Some(u) => u,
                None => continue,
            };
            match &entry.info.kind {
                OutputKind::Ndi { .. } => {
                    if let Some(sender) = &entry.ndi_sender {
                        sender.send(&frame).ok();
                    }
                }
                OutputKind::Display { .. } => {
                    if let Some(win_label) = &entry.window_label {
                        if let Some(win) = app.get_webview_window(win_label) {
                            let js = format!(
                                "var e=document.getElementById('bp-slide');if(e){{e.src='{}'}}",
                                uri
                            );
                            win.eval(&js).ok();
                        }
                    }
                }
            }
        }
    }

    /// Disconnect the presentation source and stop the compositor thread.
    pub fn disconnect_presentation_source(&self) {
        self.verse_active.store(false, Ordering::Relaxed);
        *self.presentation.lock().unwrap() = None;
    }

    /// Latest presentation frame as base64 PNG, for the operator thumbnail.
    pub fn get_presentation_preview(&self) -> Option<String> {
        let guard = self.presentation.lock().unwrap();
        let rf = guard.as_ref()?.receiver.latest_frame()?;
        drop(guard);
        let frame = Frame {
            data: rf.data,
            width: rf.width,
            height: rf.height,
        };
        let png = frame_to_png(&frame).ok()?;
        Some(B64.encode(&png))
    }

    // ── Push ──────────────────────────────────────────────────────────────────

    /// Render a Scripture verse and push to every enabled output.
    /// Sets verse_active so the compositor pauses.
    /// Returns a PNG preview (same dimensions as the configured frame).
    pub fn push_verse(
        &self,
        app: &AppHandle,
        verse_text: &str,
        reference: &str,
        cfg: &PresentConfig,
        base_frame: Option<&Frame>,
    ) -> Result<PushResult, String> {
        self.verse_active.store(true, Ordering::Relaxed);
        *self.live_verse.lock().unwrap() = Some((verse_text.to_string(), reference.to_string()));

        let scripture_full = render_frame(verse_text, reference, cfg)?;
        let scripture_overlay = if let Some(base) = base_frame {
            render_scripture_overlay(base, verse_text, reference, cfg).ok()
        } else {
            None
        };

        let layers = LayerFrames {
            base: base_frame.cloned().unwrap_or_else(|| scripture_full.clone()),
            scripture_full: Some(scripture_full.clone()),
            scripture_overlay,
            countdown: None,
        };
        self.dispatch_routed(app, &layers, cfg);

        let preview = match self.scripture_mode.lock().unwrap().clone() {
            ScriptureMode::Overlay => layers.scripture_overlay.clone().unwrap_or_else(|| scripture_full.clone()),
            ScriptureMode::Replace => scripture_full.clone(),
        };
        let png = frame_to_png(&preview)?;
        let b64 = B64.encode(&png);
        Ok(PushResult {
            png_b64: b64,
            width: preview.width,
            height: preview.height,
        })
    }

    /// Clear the live view.
    /// - If a presentation source is connected: unsets verse_active so the
    ///   compositor resumes, and returns the latest received frame as preview.
    /// - Otherwise: pushes a blank frame to all outputs.
    pub fn clear(&self, app: &AppHandle, cfg: &PresentConfig) -> Result<PushResult, String> {
        self.verse_active.store(false, Ordering::Relaxed);
        *self.live_verse.lock().unwrap() = None;

        // With a presentation source, the compositor thread resumes on its own.
        // Return the latest received frame for the operator preview thumbnail.
        {
            let guard = self.presentation.lock().unwrap();
            if let Some(src) = guard.as_ref() {
                if let Some(rf) = src.receiver.latest_frame() {
                    drop(guard);
                    let frame = Frame {
                        data: rf.data,
                        width: rf.width,
                        height: rf.height,
                    };
                    if let Ok(png) = frame_to_png(&frame) {
                        let b64 = B64.encode(&png);
                        return Ok(PushResult {
                            png_b64: b64,
                            width: frame.width,
                            height: frame.height,
                        });
                    }
                }
                // Source connected but no frame received yet.
                return Ok(PushResult {
                    png_b64: String::new(),
                    width: 0,
                    height: 0,
                });
            }
        }

        // No presentation source — push a blank frame.
        let frame = render_frame("", "", cfg)?;
        let png = frame_to_png(&frame)?;
        let b64 = B64.encode(&png);
        let layers = LayerFrames {
            base: frame.clone(),
            scripture_full: None,
            scripture_overlay: None,
            countdown: None,
        };
        self.dispatch_routed(app, &layers, cfg);
        Ok(PushResult {
            png_b64: b64,
            width: frame.width,
            height: frame.height,
        })
    }
}

