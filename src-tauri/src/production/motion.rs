use std::f32::consts::PI;

use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

use crate::present::config::{BackgroundDesign, PresentConfig, Rgba};
use crate::present::renderer::{render_background_frame, Frame};

/// Procedural motion background IDs for builtin video assets.
pub fn motion_id_for_media(media_id: &str) -> Option<&'static str> {
    match media_id {
        "blue-motion" => Some("worship_waves"),
        "conference-lines" => Some("conference_sweep"),
        "countdown-rays" => Some("light_rays"),
        _ => None,
    }
}

/// Render a time-animated motion graphic frame (CPU compositor — WebGPU-class motion without GPU deps).
pub fn render_motion_frame(
    motion_id: &str,
    design: &BackgroundDesign,
    cfg: &PresentConfig,
    time_ms: u128,
) -> Result<Frame, String> {
    let base = render_background_frame(cfg, design)?;
    let mut pixmap = frame_to_pixmap(&base)?;
    let w = cfg.width as f32;
    let h = cfg.height as f32;
    let t = time_ms as f32 / 1000.0;

    match motion_id {
        "worship_waves" => draw_waves(&mut pixmap, w, h, t),
        "conference_sweep" => draw_sweep_lines(&mut pixmap, w, h, t),
        "light_rays" => draw_light_rays(&mut pixmap, w, h, t),
        "particle_glow" => draw_particles(&mut pixmap, w, h, t),
        _ => {}
    }

    pixmap_to_frame(pixmap)
}

fn frame_to_pixmap(frame: &Frame) -> Result<Pixmap, String> {
    let mut pixmap = Pixmap::new(frame.width, frame.height).ok_or("pixmap alloc")?;
    let data = pixmap.data_mut();
    for (i, px) in frame.data.chunks(4).enumerate() {
        let idx = i * 4;
        data[idx] = px[0];
        data[idx + 1] = px[1];
        data[idx + 2] = px[2];
        data[idx + 3] = px[3];
    }
    Ok(pixmap)
}

fn pixmap_to_frame(pixmap: Pixmap) -> Result<Frame, String> {
    let w = pixmap.width();
    let h = pixmap.height();
    let raw: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let a = p.alpha();
            if a == 0 {
                return [0u8, 0, 0, 0];
            }
            let s = 255u16;
            [
                (p.red() as u16 * s / a as u16) as u8,
                (p.green() as u16 * s / a as u16) as u8,
                (p.blue() as u16 * s / a as u16) as u8,
                a,
            ]
        })
        .collect();
    Ok(Frame {
        data: raw,
        width: w,
        height: h,
    })
}

fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: &Rgba) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let mut paint = Paint::default();
    paint.set_color(tiny_skia::Color::from_rgba8(color.r, color.g, color.b, color.a));
    if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, w, h) {
        pixmap.fill_path(
            &PathBuilder::from_rect(rect),
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn draw_waves(pixmap: &mut Pixmap, w: f32, h: f32, t: f32) {
    for i in 0..6 {
        let phase = t * 0.8 + i as f32 * 0.5;
        let y_base = h * 0.55 + (i as f32 * h * 0.06);
        let alpha = (40 + i * 8).min(90) as u8;
        for x in (0..w as i32).step_by(8) {
            let xf = x as f32;
            let offset = (xf / w * PI * 4.0 + phase).sin() * h * 0.04;
            fill_rect(
                pixmap,
                xf,
                y_base + offset,
                8.0,
                3.0,
                &Rgba::new(255, 255, 255, alpha),
            );
        }
    }
}

fn draw_sweep_lines(pixmap: &mut Pixmap, w: f32, h: f32, t: f32) {
    let sweep = (t * 0.15) % 1.0;
    for i in 0..12 {
        let x = w * ((i as f32 / 12.0 + sweep) % 1.0);
        fill_rect(
            pixmap,
            x,
            0.0,
            2.0,
            h,
            &Rgba::new(255, 255, 255, 30),
        );
    }
}

fn draw_light_rays(pixmap: &mut Pixmap, w: f32, h: f32, t: f32) {
    let cx = w * 0.5;
    let cy = h * 0.35;
    for i in 0..16 {
        let angle = (i as f32 / 16.0) * PI * 2.0 + t * 0.3;
        let len = h * 0.7;
        let ex = cx + angle.cos() * len;
        let ey = cy + angle.sin() * len;
        let mut pb = PathBuilder::new();
        pb.move_to(cx, cy);
        pb.line_to(ex, ey);
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(tiny_skia::Color::from_rgba8(255, 255, 255, 25));
            let stroke = tiny_skia::Stroke {
                width: 3.0,
                ..tiny_skia::Stroke::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

fn draw_particles(pixmap: &mut Pixmap, w: f32, h: f32, t: f32) {
    for i in 0..40 {
        let seed = i as f32 * 1.618;
        let x = ((seed * 13.0 + t * 20.0) % w as f32).abs();
        let y = ((seed * 7.0 + t * 12.0) % h as f32).abs();
        let size = 2.0 + (seed % 3.0);
        fill_rect(
            pixmap,
            x,
            y,
            size,
            size,
            &Rgba::new(255, 255, 255, 80),
        );
    }
}
