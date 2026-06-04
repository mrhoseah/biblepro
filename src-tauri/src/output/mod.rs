pub mod commands;
pub mod ndi_recv;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindowBuilder, WebviewUrl};
use base64::{engine::general_purpose::STANDARD as B64, Engine};

use crate::present::config::PresentConfig;
use crate::present::ndi::NdiSender;
use crate::present::renderer::{render_frame, frame_to_png, Frame};
use ndi_recv::{NdiReceiver, ReceivedFrame};

// ── Monitor info ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub index:      usize,
    pub name:       String,
    pub width:      u32,
    pub height:     u32,
    pub x:          i32,
    pub y:          i32,
    pub is_primary: bool,
}

// ── Output types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputKind {
    Ndi     { source_name: String },
    Display { monitor_index: usize, monitor_name: String },
}

/// Serialisable status sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputInfo {
    pub id:      String,
    pub label:   String,
    pub kind:    OutputKind,
    pub enabled: bool,
    pub active:  bool,
}

/// Push result mirrors PresentResult — includes PNG preview.
#[derive(Debug, Serialize)]
pub struct PushResult {
    pub png_b64: String,
    pub width:   u32,
    pub height:  u32,
}

// ── Internal entry (not serialised) ──────────────────────────────────────────

struct OutputEntry {
    info:         OutputInfo,
    ndi_sender:   Option<NdiSender>,
    window_label: Option<String>,
}

// ── Presentation source ───────────────────────────────────────────────────────

struct PresentationSource {
    receiver:  NdiReceiver,
    // Background thread pushes presentation frames to all outputs when
    // verse_active is false. Dropping this signals the thread to stop.
    _thread:   std::thread::JoinHandle<()>,
    thread_stop: Arc<AtomicBool>,
}

impl Drop for PresentationSource {
    fn drop(&mut self) {
        self.thread_stop.store(true, Ordering::Relaxed);
    }
}

// ── OutputManager ─────────────────────────────────────────────────────────────

pub struct OutputManager {
    // Arc so the background compositor thread can hold a reference.
    outputs:      Arc<Mutex<Vec<OutputEntry>>>,
    presentation: Mutex<Option<PresentationSource>>,
    /// True while a Scripture verse is live; background compositor pauses.
    pub verse_active: Arc<AtomicBool>,
}

impl OutputManager {
    pub fn new() -> Self {
        Self {
            outputs:      Arc::new(Mutex::new(Vec::new())),
            presentation: Mutex::new(None),
            verse_active: Arc::new(AtomicBool::new(false)),
        }
    }

    // ── read ──────────────────────────────────────────────────────────────────

    pub fn get_outputs(&self) -> Vec<OutputInfo> {
        self.outputs.lock().unwrap()
            .iter()
            .map(|e| e.info.clone())
            .collect()
    }

    // ── NDI output ────────────────────────────────────────────────────────────

    pub fn add_ndi(&self, label: String, source_name: String) -> Result<OutputInfo, String> {
        let sender = NdiSender::start(&source_name)?;
        let id = uuid::Uuid::new_v4().to_string();
        let info = OutputInfo {
            id,
            label,
            kind: OutputKind::Ndi { source_name },
            enabled: true,
            active: true,
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
            kind: OutputKind::Display { monitor_index, monitor_name },
            enabled: true,
            active: true,
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
        let idx = outputs.iter().position(|e| e.info.id == id)
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
        let entry = outputs.iter_mut()
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
    /// A compositor thread starts immediately and pushes frames to all enabled
    /// outputs whenever no Scripture verse is active.
    pub fn connect_presentation_source(
        &self,
        app: AppHandle,
        source_name: String,
    ) -> Result<(), String> {
        // Stop any existing source first.
        *self.presentation.lock().unwrap() = None;

        let receiver = ndi_recv::connect_ndi_source(&source_name)?;

        let outputs_arc   = Arc::clone(&self.outputs);
        let verse_active  = Arc::clone(&self.verse_active);
        let thread_stop   = Arc::new(AtomicBool::new(false));
        let thread_stop_t = Arc::clone(&thread_stop);
        let latest        = receiver.latest.clone();

        let thread = std::thread::spawn(move || {
            let frame_interval = std::time::Duration::from_millis(33); // ~30 fps
            let idle_sleep     = std::time::Duration::from_millis(16);
            loop {
                if thread_stop_t.load(Ordering::Relaxed) { break; }

                if verse_active.load(Ordering::Relaxed) {
                    // Scripture is live — compositor pauses.
                    std::thread::sleep(idle_sleep);
                    continue;
                }

                let frame_opt = latest.lock().unwrap().clone();
                match frame_opt {
                    Some(rf) => {
                        dispatch_received(&outputs_arc, &app, &rf);
                        std::thread::sleep(frame_interval);
                    }
                    None => std::thread::sleep(idle_sleep),
                }
            }
        });

        *self.presentation.lock().unwrap() = Some(PresentationSource {
            receiver,
            _thread: thread,
            thread_stop,
        });

        Ok(())
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
        let frame = Frame { data: rf.data, width: rf.width, height: rf.height };
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
    ) -> Result<PushResult, String> {
        self.verse_active.store(true, Ordering::Relaxed);
        let frame = render_frame(verse_text, reference, cfg)?;
        let png   = frame_to_png(&frame)?;
        let b64   = B64.encode(&png);
        let data_uri = format!("data:image/png;base64,{}", b64);
        dispatch_frame(&self.outputs, app, &frame, &data_uri);
        Ok(PushResult { png_b64: b64, width: frame.width, height: frame.height })
    }

    /// Clear the live view.
    /// - If a presentation source is connected: unsets verse_active so the
    ///   compositor resumes, and returns the latest received frame as preview.
    /// - Otherwise: pushes a blank frame to all outputs.
    pub fn clear(&self, app: &AppHandle, cfg: &PresentConfig) -> Result<PushResult, String> {
        self.verse_active.store(false, Ordering::Relaxed);

        // With a presentation source, the compositor thread resumes on its own.
        // Return the latest received frame for the operator preview thumbnail.
        {
            let guard = self.presentation.lock().unwrap();
            if let Some(src) = guard.as_ref() {
                if let Some(rf) = src.receiver.latest_frame() {
                    drop(guard);
                    let frame = Frame { data: rf.data, width: rf.width, height: rf.height };
                    if let Ok(png) = frame_to_png(&frame) {
                        let b64 = B64.encode(&png);
                        return Ok(PushResult { png_b64: b64, width: frame.width, height: frame.height });
                    }
                }
                // Source connected but no frame received yet.
                return Ok(PushResult { png_b64: String::new(), width: 0, height: 0 });
            }
        }

        // No presentation source — push a blank frame.
        let frame = render_frame("", "", cfg)?;
        let png   = frame_to_png(&frame)?;
        let b64   = B64.encode(&png);
        let data_uri = format!("data:image/png;base64,{}", b64);
        dispatch_frame(&self.outputs, app, &frame, &data_uri);
        Ok(PushResult { png_b64: b64, width: frame.width, height: frame.height })
    }
}

// ── dispatch helpers ──────────────────────────────────────────────────────────

/// Push a rendered Scripture frame to all enabled outputs.
fn dispatch_frame(
    outputs: &Arc<Mutex<Vec<OutputEntry>>>,
    app: &AppHandle,
    frame: &Frame,
    data_uri: &str,
) {
    let outputs = outputs.lock().unwrap();
    for entry in outputs.iter() {
        if !entry.info.enabled { continue; }
        match &entry.info.kind {
            OutputKind::Ndi { .. } => {
                if let Some(sender) = &entry.ndi_sender {
                    sender.send(frame).ok();
                }
            }
            OutputKind::Display { .. } => {
                if let Some(win_label) = &entry.window_label {
                    if let Some(win) = app.get_webview_window(win_label) {
                        let js = format!(
                            "var e=document.getElementById('bp-slide');if(e){{e.src='{}'}}",
                            data_uri
                        );
                        win.eval(&js).ok();
                    }
                }
            }
        }
    }
}

/// Push a raw received NDI frame to all enabled outputs (compositor thread).
fn dispatch_received(
    outputs: &Arc<Mutex<Vec<OutputEntry>>>,
    app: &AppHandle,
    rf: &ReceivedFrame,
) {
    let frame = Frame { data: rf.data.clone(), width: rf.width, height: rf.height };

    // Convert to PNG once for display outputs (skip if none exist).
    let display_data_uri: Option<String> = {
        let has_display = outputs.lock().unwrap()
            .iter()
            .any(|e| e.info.enabled && matches!(e.info.kind, OutputKind::Display { .. }));
        if has_display {
            frame_to_png(&frame).ok().map(|png| {
                format!("data:image/png;base64,{}", B64.encode(&png))
            })
        } else {
            None
        }
    };

    let outputs = outputs.lock().unwrap();
    for entry in outputs.iter() {
        if !entry.info.enabled { continue; }
        match &entry.info.kind {
            OutputKind::Ndi { .. } => {
                if let Some(sender) = &entry.ndi_sender {
                    sender.send(&frame).ok();
                }
            }
            OutputKind::Display { .. } => {
                if let Some(uri) = &display_data_uri {
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
}
