use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use image::ImageReader;
use serde::{Deserialize, Serialize};

use crate::present::config::{BackgroundDesign, Rgba};
use crate::present::renderer::Frame;

use super::models::MediaDef;
use super::motion::{motion_id_for_media, render_motion_frame};
use super::themes::builtin_media;
use super::video::VideoPlayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMedia {
    pub def: MediaDef,
    pub file_path: Option<String>,
}

pub struct MediaStore {
    custom: Mutex<Vec<StoredMedia>>,
    video_cache: Mutex<HashMap<String, VideoPlayer>>,
    motion_start: Instant,
}

impl MediaStore {
    pub fn new() -> Self {
        Self {
            custom: Mutex::new(Vec::new()),
            video_cache: Mutex::new(HashMap::new()),
            motion_start: Instant::now(),
        }
    }

    pub fn load_custom(&self, items: Vec<StoredMedia>) {
        *self.custom.lock().unwrap() = items;
    }

    pub fn custom_items(&self) -> Vec<StoredMedia> {
        self.custom.lock().unwrap().clone()
    }

    pub fn motion_time_ms(&self) -> u128 {
        self.motion_start.elapsed().as_millis()
    }

    pub fn custom_count(&self) -> usize {
        self.custom.lock().unwrap().len()
    }

    pub fn all_media(&self) -> Vec<MediaDef> {
        let mut items: Vec<MediaDef> = builtin_media();
        items.extend(self.custom.lock().unwrap().iter().map(|s| s.def.clone()));
        items
    }

    pub fn get(&self, id: &str) -> Option<StoredMedia> {
        if let Some(mut def) = builtin_media().into_iter().find(|m| m.id == id) {
            if def.motion_id.is_none() {
                def.motion_id = motion_id_for_media(id).map(str::to_string);
            }
            return Some(StoredMedia {
                def,
                file_path: None,
            });
        }
        self.custom
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.def.id == id)
            .cloned()
    }

    pub fn import_image(
        &self,
        app_dir: &Path,
        source_path: &str,
        title: String,
        category: String,
    ) -> Result<MediaDef, String> {
        self.import_file(app_dir, source_path, title, category, "image")
    }

    pub fn import_video(
        &self,
        app_dir: &Path,
        source_path: &str,
        title: String,
        category: String,
    ) -> Result<MediaDef, String> {
        self.import_file(app_dir, source_path, title, category, "video")
    }

    fn import_file(
        &self,
        app_dir: &Path,
        source_path: &str,
        title: String,
        category: String,
        media_type: &str,
    ) -> Result<MediaDef, String> {
        let media_dir = app_dir.join("media");
        std::fs::create_dir_all(&media_dir).map_err(|e| e.to_string())?;

        let id = format!("import-{}", uuid::Uuid::new_v4());
        let ext = Path::new(source_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let dest = media_dir.join(format!("{id}.{ext}"));
        std::fs::copy(source_path, &dest).map_err(|e| e.to_string())?;
        let dest_str = dest.to_string_lossy().into_owned();

        let def = MediaDef {
            id: id.clone(),
            title,
            category,
            media_type: media_type.into(),
            background: BackgroundDesign::solid(Rgba::black()),
            motion_id: None,
        };

        self.custom.lock().unwrap().push(StoredMedia {
            def: def.clone(),
            file_path: Some(dest_str),
        });

        Ok(def)
    }

    pub fn render_stored(
        &self,
        stored: &StoredMedia,
        width: u32,
        height: u32,
    ) -> Result<Frame, String> {
        let cfg = default_cfg(width, height);

        if let Some(path) = &stored.file_path {
            let mut cache = self.video_cache.lock().unwrap();
            if !cache.contains_key(&stored.def.id) {
                if let Ok(player) = VideoPlayer::from_path(path, width, height) {
                    cache.insert(stored.def.id.clone(), player);
                }
            }
            if let Some(player) = cache.get(&stored.def.id) {
                return Ok(player.current_frame().clone());
            }
            return render_image_file(path, width, height);
        }

        if stored.def.media_type == "video" {
            let motion_id = stored
                .def
                .motion_id
                .as_deref()
                .or_else(|| motion_id_for_media(&stored.def.id));
            if let Some(mid) = motion_id {
                return render_motion_frame(mid, &stored.def.background, &cfg, self.motion_time_ms());
            }
        }

        crate::production::renderer::render_media_frame(&stored.def.background, &cfg)
    }
}

fn default_cfg(width: u32, height: u32) -> crate::present::config::PresentConfig {
    let mut cfg = crate::present::config::PresentConfig::default();
    cfg.width = width;
    cfg.height = height;
    cfg
}

pub fn render_image_file(path: &str, width: u32, height: u32) -> Result<Frame, String> {
    let img = ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (sw, sh) = (rgba.width(), rgba.height());

    let mut out = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let sx = (x as f32 / width as f32 * sw as f32) as u32;
            let sy = (y as f32 / height as f32 * sh as f32) as u32;
            let px = rgba.get_pixel(sx.min(sw - 1), sy.min(sh - 1));
            let i = ((y * width + x) * 4) as usize;
            out[i] = px[0];
            out[i + 1] = px[1];
            out[i + 2] = px[2];
            out[i + 3] = px[3];
        }
    }

    Ok(Frame {
        data: out,
        width,
        height,
    })
}
