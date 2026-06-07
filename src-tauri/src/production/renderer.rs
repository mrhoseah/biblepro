use ab_glyph::{Font, FontRef, PxScale, PxScaleFont, ScaleFont};
use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Point, Stroke, Transform,
};

use crate::present::config::{BackgroundDesign, PresentConfig, Rgba};
use crate::present::renderer::{render_background_frame, Frame};

use super::models::{CountdownRuntime, CountdownStyle, ProductionTheme};
use super::themes::media_by_id;

static FONT_BOLD: &[u8] = include_bytes!("../../resources/fonts/Lora-Bold.ttf");
static FONT_REGULAR: &[u8] = include_bytes!("../../resources/fonts/Lora-Regular.ttf");

fn to_color(c: &Rgba) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

fn load_bold(size: f32) -> Result<PxScaleFont<FontRef<'static>>, String> {
    FontRef::try_from_slice(FONT_BOLD)
        .map(|f| f.into_scaled(PxScale::from(size)))
        .map_err(|e| e.to_string())
}

fn load_regular(size: f32) -> Result<PxScaleFont<FontRef<'static>>, String> {
    FontRef::try_from_slice(FONT_REGULAR)
        .map(|f| f.into_scaled(PxScale::from(size)))
        .map_err(|e| e.to_string())
}

fn paint_text(
    pixmap: &mut Pixmap,
    font: &PxScaleFont<FontRef<'_>>,
    text: &str,
    cx: f32,
    baseline: f32,
    color: &Rgba,
) {
    let x = cx - measure_center(font, text) / 2.0;
    paint_line(pixmap, font, text, x, baseline, color);
}

fn paint_line<F: Font>(
    pixmap: &mut Pixmap,
    font: &PxScaleFont<F>,
    text: &str,
    x: f32,
    baseline_y: f32,
    color: &Rgba,
) {
    let mut cx = x;
    let mut last = None;
    for c in text.chars() {
        let id = font.glyph_id(c);
        if let Some(prev) = last {
            cx += font.kern(prev, id);
        }
        let glyph = id.with_scale_and_position(font.scale(), ab_glyph::point(cx, baseline_y));
        if let Some(og) = font.outline_glyph(glyph) {
            let b = og.px_bounds();
            og.draw(|gx, gy, cov| {
                let px = b.min.x as i32 + gx as i32;
                let py = b.min.y as i32 + gy as i32;
                if px < 0 || py < 0 {
                    return;
                }
                let (px, py) = (px as u32, py as u32);
                if px >= pixmap.width() || py >= pixmap.height() {
                    return;
                }
                let a = (cov * color.a as f32) as u16;
                let inv = 255 - a;
                let i = (py * pixmap.width() + px) as usize * 4;
                let d = pixmap.data_mut();
                d[i] = ((color.r as u16 * a + d[i] as u16 * inv) / 255) as u8;
                d[i + 1] = ((color.g as u16 * a + d[i + 1] as u16 * inv) / 255) as u8;
                d[i + 2] = ((color.b as u16 * a + d[i + 2] as u16 * inv) / 255) as u8;
                d[i + 3] = (a + d[i + 3] as u16 * inv / 255) as u8;
            });
        }
        cx += font.h_advance(id);
        last = Some(id);
    }
}

fn measure_center<F: Font>(font: &PxScaleFont<F>, text: &str) -> f32 {
    let mut x = 0.0f32;
    let mut last = None;
    for c in text.chars() {
        let id = font.glyph_id(c);
        if let Some(prev) = last {
            x += font.kern(prev, id);
        }
        x += font.h_advance(id);
        last = Some(id);
    }
    x
}

fn format_time(secs: u32) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{m:02}:{s:02}")
}

fn resolve_background(
    theme: &ProductionTheme,
    media_id: Option<&str>,
) -> BackgroundDesign {
    if let Some(id) = media_id {
        if let Some(media) = media_by_id(id) {
            return media.background;
        }
    }
    theme.background.clone()
}

fn draw_ring(
    pixmap: &mut Pixmap,
    cx: f32,
    cy: f32,
    radius: f32,
    progress: f32,
    color: &Rgba,
) {
    let stroke_width = radius * 0.08;
    let track = Rgba::new(color.r, color.g, color.b, 40);
    let mut track_paint = Paint::default();
    track_paint.set_color(to_color(&track));
    track_paint.anti_alias = true;

    let mut pb = PathBuilder::new();
    pb.push_circle(cx, cy, radius);
    if let Some(path) = pb.finish() {
        let stroke = Stroke {
            width: stroke_width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        pixmap.stroke_path(&path, &track_paint, &stroke, Transform::identity(), None);
    }

    if progress <= 0.0 {
        return;
    }

    let sweep = progress * 360.0;
    let start = -90.0f32;
    let steps = (sweep / 4.0).max(1.0) as usize;
    let mut pb = PathBuilder::new();
    let to_rad = |deg: f32| deg.to_radians();
    let point = |deg: f32| {
        Point::from_xy(
            cx + radius * to_rad(deg).cos(),
            cy + radius * to_rad(deg).sin(),
        )
    };
    pb.move_to(point(start).x, point(start).y);
    for i in 1..=steps {
        let deg = start + sweep * i as f32 / steps as f32;
        pb.line_to(point(deg).x, point(deg).y);
    }

    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(to_color(color));
        paint.anti_alias = true;
        let stroke = Stroke {
            width: stroke_width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn draw_loader_bar(pixmap: &mut Pixmap, w: f32, h: f32, progress: f32, loader: &str, color: &Rgba) {
    let bar_y = h * 0.92;
    let bar_h = h * 0.012;
    let pad = w * 0.12;
    let bar_w = w - pad * 2.0;

    let track = Rgba::new(255, 255, 255, 40);
    fill_rect(pixmap, pad, bar_y, bar_w, bar_h, &track);

    if loader.contains("Progress") || loader.contains("Minimal") || loader.contains("Segments") {
        fill_rect(pixmap, pad, bar_y, bar_w * progress, bar_h, color);
    }

    if loader.contains("Wave") {
        let count = 18;
        for i in 0..count {
            let x = pad + (bar_w / count as f32) * i as f32;
            let height = bar_h * 4.0 + (i % 5) as f32 * bar_h * 2.0;
            let active = (i as f32 / count as f32) <= progress;
            let c = if active {
                color.clone()
            } else {
                Rgba::new(color.r, color.g, color.b, 60)
            };
            fill_rect(pixmap, x, bar_y - height, bar_w / count as f32 * 0.6, height, &c);
        }
    }
}

fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: &Rgba) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let mut paint = Paint::default();
    paint.set_color(to_color(color));
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

fn draw_pulse(pixmap: &mut Pixmap, cx: f32, cy: f32, radius: f32, color: &Rgba) {
    let mut pb = PathBuilder::new();
    pb.push_circle(cx, cy, radius);
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(to_color(&Rgba::new(color.r, color.g, color.b, 30)));
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

        let stroke = Stroke {
            width: 2.0,
            ..Stroke::default()
        };
        paint.set_color(to_color(&Rgba::new(color.r, color.g, color.b, 80)));
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn frame_to_pixmap(frame: &Frame) -> Result<Pixmap, String> {
    let mut pixmap = Pixmap::new(frame.width, frame.height).ok_or("pixmap alloc")?;
    let data = pixmap.data_mut();
    for (i, px) in frame.data.chunks(4).enumerate() {
        let a = px[3];
        let idx = i * 4;
        data[idx] = px[0];
        data[idx + 1] = px[1];
        data[idx + 2] = px[2];
        data[idx + 3] = a;
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

/// Composite a countdown scene: media/theme background + timer overlay.
pub fn render_countdown_frame(
    countdown: &CountdownRuntime,
    theme: &ProductionTheme,
    cfg: &PresentConfig,
) -> Result<Frame, String> {
    let bg = resolve_background(theme, countdown.def.media_id.as_deref());
    let base = render_background_frame(cfg, &bg)?;

    let w = cfg.width as f32;
    let h = cfg.height as f32;
    let cx = w * 0.5;
    let cy = h * 0.5;

    let duration = countdown.def.duration.max(1) as f32;
    let remaining = countdown.remaining_secs as f32;
    let progress = (remaining / duration).clamp(0.0, 1.0);
    let time_str = format_time(countdown.remaining_secs);

    let mut pixmap = frame_to_pixmap(&base)?;

    // Subtle darken overlay for readability
    fill_rect(
        &mut pixmap,
        0.0,
        0.0,
        w,
        h,
        &Rgba::new(0, 0, 0, 60),
    );

    let headline_font = load_bold(h * 0.035)?;
    let timer_font = load_bold(h * 0.14)?;
    let sub_font = load_regular(h * 0.028)?;

    let headline_y = cy - h * 0.12;
    let timer_y = cy + h * 0.04;
    let sub_y = cy + h * 0.16;

    paint_text(
        &mut pixmap,
        &headline_font,
        &countdown.def.headline,
        cx,
        headline_y,
        &theme.headline_color,
    );
    paint_text(
        &mut pixmap,
        &timer_font,
        &time_str,
        cx,
        timer_y,
        &theme.timer_color,
    );
    paint_text(
        &mut pixmap,
        &sub_font,
        &countdown.def.subline,
        cx,
        sub_y,
        &theme.subline_color,
    );

    match countdown.def.style {
        CountdownStyle::Ring | CountdownStyle::Theme => {
            draw_ring(&mut pixmap, cx, cy, h * 0.22, progress, &theme.timer_color);
        }
        CountdownStyle::Loader => {
            if countdown.def.loader.contains("Pulse") {
                draw_pulse(&mut pixmap, cx, cy, h * 0.26, &theme.timer_color);
            }
            draw_loader_bar(
                &mut pixmap,
                w,
                h,
                progress,
                &countdown.def.loader,
                &theme.timer_color,
            );
        }
        CountdownStyle::Numeric => {
            draw_loader_bar(
                &mut pixmap,
                w,
                h,
                progress,
                &countdown.def.loader,
                &theme.timer_color,
            );
        }
    }

    pixmap_to_frame(pixmap)
}

/// Render a media-only background frame.
pub fn render_media_frame(design: &BackgroundDesign, cfg: &PresentConfig) -> Result<Frame, String> {
    render_background_frame(cfg, design)
}
