use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use gif::{ColorOutput, DecodeOptions};

use crate::present::renderer::Frame;

use super::media_store::render_image_file;

/// Looped video/GIF frame player for the live compositor.
pub struct VideoPlayer {
    frames: Vec<Frame>,
    frame_duration_ms: u32,
    started: Instant,
}

impl VideoPlayer {
    pub fn from_gif(path: &str, width: u32, height: u32) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut opts = DecodeOptions::new();
        opts.set_color_output(ColorOutput::RGBA);
        let mut reader = opts.read_info(file).map_err(|e| e.to_string())?;
        let mut frames = Vec::new();
        let mut total_delay_ms = 0u32;
        let mut delay_count = 0u32;

        loop {
            match reader.read_next_frame() {
                Ok(Some(frame)) => {
                    let delay_ms = (frame.delay as u32).saturating_mul(10).max(20);
                    total_delay_ms += delay_ms;
                    delay_count += 1;
                    let rgba = resize_rgba(
                        &frame.buffer,
                        frame.width as u32,
                        frame.height as u32,
                        width,
                        height,
                    );
                    frames.push(Frame {
                        data: rgba,
                        width,
                        height,
                    });
                }
                Ok(None) => break,
                Err(e) => return Err(e.to_string()),
            }
        }

        if frames.is_empty() {
            return Err("GIF contains no frames".into());
        }

        let frame_duration_ms = if delay_count > 0 {
            (total_delay_ms / delay_count).max(20)
        } else {
            100
        };

        Ok(Self {
            frames,
            frame_duration_ms,
            started: Instant::now(),
        })
    }

    pub fn from_mp4_ffmpeg(path: &str, width: u32, height: u32) -> Result<Self, String> {
        let mut child = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-i",
                path,
                "-vf",
                &format!("scale={width}:{height},fps=30"),
                "-t",
                "30",
                "-pix_fmt",
                "rgba",
                "-f",
                "rawvideo",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| "ffmpeg not found — install ffmpeg for MP4 playback".to_string())?;

        let frame_size = (width * height * 4) as usize;
        let mut stdout = child.stdout.take().ok_or("ffmpeg stdout unavailable")?;
        let mut frames = Vec::new();
        let mut buf = vec![0u8; frame_size];

        while stdout.read_exact(&mut buf).is_ok() {
            frames.push(Frame {
                data: buf.clone(),
                width,
                height,
            });
        }
        let _ = child.wait();

        if frames.is_empty() {
            let frame = extract_poster_frame(path, width, height)?;
            return Ok(Self {
                frames: vec![frame],
                frame_duration_ms: 1000,
                started: Instant::now(),
            });
        }

        Ok(Self {
            frames,
            frame_duration_ms: 33,
            started: Instant::now(),
        })
    }

    pub fn from_path(path: &str, width: u32, height: u32) -> Result<Self, String> {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "gif" => Self::from_gif(path, width, height),
            "mp4" | "webm" | "mov" | "mkv" => Self::from_mp4_ffmpeg(path, width, height),
            "png" | "jpg" | "jpeg" | "webp" => {
                let frame = render_image_file(path, width, height)?;
                Ok(Self {
                    frames: vec![frame],
                    frame_duration_ms: 1000,
                    started: Instant::now(),
                })
            }
            _ => Err(format!("Unsupported video format: {ext}")),
        }
    }

    pub fn current_frame(&self) -> &Frame {
        if self.frames.len() == 1 {
            return &self.frames[0];
        }
        let elapsed = self.started.elapsed().as_millis() as u32;
        let total = self.frames.len() as u32 * self.frame_duration_ms.max(1);
        let pos = elapsed % total;
        let index = (pos / self.frame_duration_ms.max(1)) as usize;
        &self.frames[index.min(self.frames.len() - 1)]
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

fn resize_rgba(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for y in 0..dh {
        for x in 0..dw {
            let sx = (x as f32 / dw as f32 * sw as f32) as u32;
            let sy = (y as f32 / dh as f32 * sh as f32) as u32;
            let si = ((sy * sw + sx) * 4) as usize;
            let di = ((y * dw + x) * 4) as usize;
            if si + 3 < src.len() {
                out[di] = src[si];
                out[di + 1] = src[si + 1];
                out[di + 2] = src[si + 2];
                out[di + 3] = src[si + 3];
            }
        }
    }
    out
}

fn extract_poster_frame(path: &str, width: u32, height: u32) -> Result<Frame, String> {
    let out_path = std::env::temp_dir().join(format!("bp_poster_{}.png", uuid::Uuid::new_v4()));
    let out_str = out_path.to_string_lossy().to_string();
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            path,
            "-vframes",
            "1",
            "-vf",
            &format!("scale={width}:{height}"),
            &out_str,
        ])
        .status()
        .map_err(|_| "ffmpeg not available")?;
    if !status.success() {
        return Err("ffmpeg poster extraction failed".into());
    }
    let frame = render_image_file(&out_str, width, height)?;
    let _ = std::fs::remove_file(&out_path);
    Ok(frame)
}
