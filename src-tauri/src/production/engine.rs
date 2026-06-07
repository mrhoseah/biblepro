use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use tauri::{AppHandle, Manager};

use crate::output::routing::LayerFrames;
use crate::output::OutputManager;
use crate::present::config::PresentConfig;
use crate::present::renderer::{frame_to_png, render_frame, render_scripture_overlay, Frame};

use super::media_store::MediaStore;
use super::models::{
    CountdownDef, CountdownRuntime, CountdownSchedule, CountdownStatus, ProductionPreview,
    ProductionSnapshot, TransitionTarget,
};
use super::scheduler::{schedule_status, should_fire};
use super::packs::{export_countdown_pack, import_countdown_pack};
use super::renderer::render_countdown_frame;
use super::themes::{builtin_countdowns, theme_by_id};

struct CountdownClock {
    started_at: Instant,
    paused_remaining: u32,
}

struct ProductionInner {
    countdown: Option<CountdownRuntime>,
    countdown_clock: Option<CountdownClock>,
    current_media_id: Option<String>,
    media_live: bool,
    auto_transition: bool,
    transition_target: TransitionTarget,
    custom_countdowns: Vec<CountdownDef>,
    schedule: CountdownSchedule,
}

pub struct ProductionManager {
    inner: Mutex<ProductionInner>,
    compositor_started: AtomicBool,
    pub media_store: MediaStore,
}

impl ProductionManager {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(ProductionInner {
                countdown: None,
                countdown_clock: None,
                current_media_id: None,
                media_live: false,
                auto_transition: true,
                transition_target: TransitionTarget::Media,
                custom_countdowns: Vec::new(),
                schedule: CountdownSchedule::default(),
            }),
            compositor_started: AtomicBool::new(false),
            media_store: MediaStore::new(),
        }
    }

    pub fn start_compositor(app: AppHandle) {
        let production = app.state::<ProductionManager>();
        if production.compositor_started.swap(true, Ordering::Relaxed) {
            return;
        }

        std::thread::spawn(move || {
            let frame_interval = Duration::from_millis(33);
            let idle_sleep = Duration::from_millis(50);
            loop {
                let production = app.state::<ProductionManager>();
                let outputs = app.state::<OutputManager>();
                let present = app.state::<crate::present::PresentState>();

                if !needs_compositor(&production, &outputs) {
                    std::thread::sleep(idle_sleep);
                    continue;
                }

                let cfg = present.config.lock().unwrap().clone();

                // Tick countdown + auto-transition
                {
                    let ended = {
                        let tick = {
                            let inner = production.inner.lock().unwrap();
                            inner.countdown_clock.as_ref().map(|c| {
                                (c.started_at, c.paused_remaining)
                            })
                        };
                        let mut inner = production.inner.lock().unwrap();
                        let mut ended_now = false;
                        if let (Some(cd), Some((started_at, paused_remaining))) =
                            (inner.countdown.as_mut(), tick)
                        {
                            if cd.status == CountdownStatus::Running {
                                let elapsed = started_at.elapsed().as_secs() as u32;
                                let remaining = paused_remaining.saturating_sub(elapsed);
                                cd.remaining_secs = remaining;
                                if remaining == 0 {
                                    cd.status = CountdownStatus::Ended;
                                    inner.countdown_clock = None;
                                    ended_now = true;
                                }
                            }
                        }
                        ended_now
                    };
                    if ended {
                        production.apply_auto_transition(&outputs);
                    }
                }

                production.check_schedule(&outputs);

                if let Ok(layers) = production.build_layers(&outputs, &cfg) {
                    outputs.dispatch_routed(&app, &layers, &cfg);
                    let _ = frame_to_png(&layers.base).ok(); // warm png path
                }

                std::thread::sleep(frame_interval);
            }
        });
    }

    fn apply_auto_transition(&self, outputs: &OutputManager) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.auto_transition {
            return;
        }
        let media_id = inner
            .countdown
            .as_ref()
            .and_then(|c| c.def.media_id.clone());
        match inner.transition_target {
            TransitionTarget::Stop => {
                if let Some(cd) = inner.countdown.as_mut() {
                    cd.status = CountdownStatus::Idle;
                    cd.remaining_secs = cd.def.duration;
                }
            }
            TransitionTarget::Idle => {
                inner.media_live = false;
                if let Some(cd) = inner.countdown.as_mut() {
                    cd.status = CountdownStatus::Idle;
                }
            }
            TransitionTarget::Media => {
                if let Some(id) = media_id {
                    inner.current_media_id = Some(id);
                    inner.media_live = true;
                }
                if let Some(cd) = inner.countdown.as_mut() {
                    cd.status = CountdownStatus::Idle;
                }
            }
        }
        inner.countdown_clock = None;
        let _ = outputs;
    }

    pub fn build_layers(
        &self,
        outputs: &OutputManager,
        cfg: &PresentConfig,
    ) -> Result<LayerFrames, String> {
        let inner = self.inner.lock().unwrap();
        let base = self.compose_base_frame_inner(&inner, outputs, cfg)?;
        let countdown = if inner.countdown.as_ref().is_some_and(|cd| {
            matches!(
                cd.status,
                CountdownStatus::Running | CountdownStatus::Paused | CountdownStatus::Ended
            )
        }) {
            inner
                .countdown
                .as_ref()
                .map(|cd| {
                    let theme = theme_by_id(&cd.def.theme_id);
                    render_countdown_frame(cd, &theme, cfg)
                })
                .transpose()?
        } else {
            None
        };

        let (scripture_full, scripture_overlay) = {
            let live = outputs.live_verse.lock().unwrap().clone();
            if let Some((text, reference)) = live {
                let full = render_frame(&text, &reference, cfg)?;
                let overlay = render_scripture_overlay(&base, &text, &reference, cfg).ok();
                (Some(full), overlay)
            } else {
                (None, None)
            }
        };

        Ok(LayerFrames {
            base: countdown.clone().unwrap_or(base),
            scripture_full,
            scripture_overlay,
            countdown,
        })
    }

    pub fn compose_base_for_push(
        &self,
        outputs: &OutputManager,
        cfg: &PresentConfig,
    ) -> Result<Frame, String> {
        let inner = self.inner.lock().unwrap();
        self.compose_base_frame_inner(&inner, outputs, cfg)
    }

    fn compose_base_frame_inner(
        &self,
        inner: &ProductionInner,
        outputs: &OutputManager,
        cfg: &PresentConfig,
    ) -> Result<Frame, String> {
        if let Some(cd) = inner.countdown.as_ref() {
            if matches!(
                cd.status,
                CountdownStatus::Running | CountdownStatus::Paused | CountdownStatus::Ended
            ) {
                let theme = theme_by_id(&cd.def.theme_id);
                return render_countdown_frame(cd, &theme, cfg);
            }
        }

        if let Some(rf) = outputs.latest_presentation_frame() {
            return Ok(Frame {
                data: rf.data,
                width: rf.width,
                height: rf.height,
            });
        }

        if inner.media_live {
            if let Some(id) = &inner.current_media_id {
                if let Some(stored) = self.media_store.get(id) {
                    return self
                        .media_store
                        .render_stored(&stored, cfg.width, cfg.height);
                }
            }
        }

        render_frame("", "", cfg)
    }

    pub fn snapshot(&self, outputs: &OutputManager) -> ProductionSnapshot {
        let inner = self.inner.lock().unwrap();
        self.snapshot_from_inner(&inner, outputs)
    }

    fn snapshot_from_inner(
        &self,
        inner: &ProductionInner,
        outputs: &OutputManager,
    ) -> ProductionSnapshot {
        let routing = outputs.routing_snapshot();
        ProductionSnapshot {
            countdown: inner.countdown.clone(),
            current_media_id: inner.current_media_id.clone(),
            media_live: inner.media_live,
            presentation_connected: outputs.has_presentation_source(),
            active_layer: active_layer_name(inner, outputs),
            auto_transition: inner.auto_transition,
            transition_target: inner.transition_target.clone(),
            scripture_mode: match routing.scripture_mode {
                crate::output::routing::ScriptureMode::Replace => "replace".into(),
                crate::output::routing::ScriptureMode::Overlay => "overlay".into(),
            },
            custom_countdown_count: inner.custom_countdowns.len(),
            custom_media_count: self.media_store.custom_count(),
            schedule: schedule_status(&inner.schedule),
        }
    }

    fn check_schedule(&self, outputs: &OutputManager) {
        let fire = {
            let inner = self.inner.lock().unwrap();
            should_fire(&inner.schedule)
        };
        if !fire {
            return;
        }
        let countdown_id = {
            let mut inner = self.inner.lock().unwrap();
            if inner.schedule.fired {
                return;
            }
            let id = inner.schedule.countdown_id.clone();
            inner.schedule.fired = true;
            id
        };
        let _ = self.set_countdown(&countdown_id, outputs);
        let _ = self.start_countdown(outputs);
    }

    pub fn all_countdowns(&self) -> Vec<CountdownDef> {
        let mut items = builtin_countdowns();
        items.extend(self.inner.lock().unwrap().custom_countdowns.clone());
        items
    }

    pub fn set_countdown(
        &self,
        id: &str,
        outputs: &OutputManager,
    ) -> Result<ProductionSnapshot, String> {
        let def = self
            .all_countdowns()
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| format!("Countdown '{id}' not found"))?;
        let mut inner = self.inner.lock().unwrap();
        inner.countdown = Some(CountdownRuntime {
            def: def.clone(),
            status: CountdownStatus::Idle,
            remaining_secs: def.duration,
        });
        inner.countdown_clock = None;
        Ok(self.snapshot(outputs))
    }

    pub fn start_countdown(&self, outputs: &OutputManager) -> Result<ProductionSnapshot, String> {
        let mut inner = self.inner.lock().unwrap();
        let cd = inner
            .countdown
            .as_mut()
            .ok_or("No countdown loaded — select one first")?;
        cd.status = CountdownStatus::Running;
        cd.remaining_secs = cd.def.duration;
        inner.countdown_clock = Some(CountdownClock {
            started_at: Instant::now(),
            paused_remaining: cd.def.duration,
        });
        Ok(self.snapshot(outputs))
    }

    pub fn pause_countdown(&self, outputs: &OutputManager) -> Result<ProductionSnapshot, String> {
        let mut inner = self.inner.lock().unwrap();
        let remaining = inner.countdown_clock.as_ref().map(|clock| {
            let elapsed = clock.started_at.elapsed().as_secs() as u32;
            clock.paused_remaining.saturating_sub(elapsed)
        });
        let cd = inner.countdown.as_mut().ok_or("No countdown loaded")?;
        if cd.status != CountdownStatus::Running {
            return Ok(self.snapshot(outputs));
        }
        if let Some(rem) = remaining {
            cd.remaining_secs = rem;
        }
        cd.status = CountdownStatus::Paused;
        inner.countdown_clock = None;
        Ok(self.snapshot(outputs))
    }

    pub fn resume_countdown(&self, outputs: &OutputManager) -> Result<ProductionSnapshot, String> {
        let mut inner = self.inner.lock().unwrap();
        let cd = inner.countdown.as_mut().ok_or("No countdown loaded")?;
        if cd.status != CountdownStatus::Paused {
            return Ok(self.snapshot(outputs));
        }
        cd.status = CountdownStatus::Running;
        inner.countdown_clock = Some(CountdownClock {
            started_at: Instant::now(),
            paused_remaining: cd.remaining_secs,
        });
        Ok(self.snapshot(outputs))
    }

    pub fn stop_countdown(&self, outputs: &OutputManager) -> Result<ProductionSnapshot, String> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(cd) = inner.countdown.as_mut() {
            cd.status = CountdownStatus::Idle;
            cd.remaining_secs = cd.def.duration;
        }
        inner.countdown_clock = None;
        Ok(self.snapshot(outputs))
    }

    pub fn set_media(
        &self,
        id: &str,
        outputs: &OutputManager,
    ) -> Result<ProductionSnapshot, String> {
        self.media_store
            .get(id)
            .ok_or_else(|| format!("Media '{id}' not found"))?;
        let mut inner = self.inner.lock().unwrap();
        inner.current_media_id = Some(id.to_string());
        Ok(self.snapshot(outputs))
    }

    pub fn set_media_live(
        &self,
        live: bool,
        outputs: &OutputManager,
    ) -> Result<ProductionSnapshot, String> {
        let mut inner = self.inner.lock().unwrap();
        if live && inner.current_media_id.is_none() {
            return Err("Select media before going live".into());
        }
        inner.media_live = live;
        Ok(self.snapshot(outputs))
    }

    pub fn set_auto_transition(
        &self,
        enabled: bool,
        target: TransitionTarget,
        outputs: &OutputManager,
    ) -> Result<ProductionSnapshot, String> {
        let snap = {
            let mut inner = self.inner.lock().unwrap();
            inner.auto_transition = enabled;
            inner.transition_target = target;
            self.snapshot_from_inner(&inner, outputs)
        };
        Ok(snap)
    }

    pub fn set_countdown_schedule(
        &self,
        schedule: CountdownSchedule,
        outputs: &OutputManager,
    ) -> Result<ProductionSnapshot, String> {
        let snap = {
            let mut inner = self.inner.lock().unwrap();
            inner.schedule = schedule;
            self.snapshot_from_inner(&inner, outputs)
        };
        Ok(snap)
    }

    pub fn import_pack(
        &self,
        json: &str,
        outputs: &OutputManager,
    ) -> Result<ProductionSnapshot, String> {
        let def = import_countdown_pack(json)?;
        let mut inner = self.inner.lock().unwrap();
        inner.custom_countdowns.retain(|c| c.id != def.id);
        inner.custom_countdowns.push(def);
        Ok(self.snapshot(outputs))
    }

    pub fn export_pack(&self, id: &str) -> Result<String, String> {
        if self
            .inner
            .lock()
            .unwrap()
            .custom_countdowns
            .iter()
            .any(|c| c.id == id)
        {
            let def = self
                .all_countdowns()
                .into_iter()
                .find(|c| c.id == id)
                .ok_or_else(|| format!("Countdown '{id}' not found"))?;
            let theme_id = def.theme_id.clone();
            return super::packs::CountdownPack::new(def, Some(theme_by_id(&theme_id)))
                .export_json();
        }
        export_countdown_pack(id)
    }

    pub fn import_media_file(
        &self,
        app_dir: &std::path::Path,
        path: &str,
        title: String,
        category: String,
        outputs: &OutputManager,
    ) -> Result<ProductionSnapshot, String> {
        self.media_store
            .import_image(app_dir, path, title, category)?;
        Ok(self.snapshot(outputs))
    }

    pub fn import_video_file(
        &self,
        app_dir: &std::path::Path,
        path: &str,
        title: String,
        category: String,
        outputs: &OutputManager,
    ) -> Result<ProductionSnapshot, String> {
        self.media_store
            .import_video(app_dir, path, title, category)?;
        Ok(self.snapshot(outputs))
    }

    pub fn preview(
        &self,
        outputs: &OutputManager,
        cfg: &PresentConfig,
    ) -> Result<ProductionPreview, String> {
        let layers = self.build_layers(outputs, cfg)?;
        let preview_frame = layers
            .scripture_overlay
            .clone()
            .or(layers.scripture_full.clone())
            .unwrap_or_else(|| layers.base.clone());
        let png = frame_to_png(&preview_frame)?;
        Ok(ProductionPreview {
            png_b64: B64.encode(&png),
            width: preview_frame.width,
            height: preview_frame.height,
            snapshot: self.snapshot(outputs),
        })
    }

}

fn needs_compositor(production: &ProductionManager, outputs: &OutputManager) -> bool {
    let inner = production.inner.lock().unwrap();
    let countdown_active = inner.countdown.as_ref().is_some_and(|cd| {
        matches!(
            cd.status,
            CountdownStatus::Running | CountdownStatus::Paused | CountdownStatus::Ended
        )
    });
    let has_outputs = !outputs.get_outputs().is_empty();
    countdown_active
        || inner.media_live
        || outputs.has_presentation_source()
        || outputs.live_verse.lock().unwrap().is_some()
        || has_outputs
}

fn active_layer_name(inner: &ProductionInner, outputs: &OutputManager) -> String {
    if outputs.live_verse.lock().unwrap().is_some() {
        return "scripture".into();
    }
    if let Some(cd) = &inner.countdown {
        if matches!(
            cd.status,
            CountdownStatus::Running | CountdownStatus::Paused | CountdownStatus::Ended
        ) {
            return "countdown".into();
        }
    }
    if outputs.has_presentation_source() {
        return "presentation".into();
    }
    if inner.media_live {
        return "media".into();
    }
    "idle".into()
}
