use ab_glyph::{Font, FontRef, PxScale, PxScaleFont, ScaleFont};
use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder,
    Pixmap, Point, RadialGradient, SpreadMode, Transform,
};
use super::config::{BackgroundDesign, BgMode, PresentConfig, Rgba, Template, TextPosition};

static FONT_BOLD:    &[u8] = include_bytes!("../../resources/fonts/Lora-Bold.ttf");
static FONT_REGULAR: &[u8] = include_bytes!("../../resources/fonts/Lora-Regular.ttf");

pub struct Frame {
    pub data:   Vec<u8>,
    pub width:  u32,
    pub height: u32,
}

// ── colour helpers ────────────────────────────────────────────────────────────

fn to_color(c: &Rgba) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: &Rgba) {
    if w <= 0.0 || h <= 0.0 { return; }
    let mut paint = Paint::default();
    paint.set_color(to_color(color));
    paint.anti_alias = false;
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

fn bg_stops(design: &BackgroundDesign) -> Vec<GradientStop> {
    design.stops.iter().map(|s| GradientStop::new(s.pos.clamp(0.0, 1.0), to_color(&s.color))).collect()
}

/// Fill a rectangle using a `BackgroundDesign` (gradient or solid).
fn fill_design(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, design: &BackgroundDesign) {
    if w <= 0.0 || h <= 0.0 { return; }
    let rect = match tiny_skia::Rect::from_xywh(x, y, w, h) { Some(r) => r, None => return };
    let stops = bg_stops(design);
    if stops.is_empty() { return; }

    let shader = match &design.mode {
        BgMode::Solid => {
            fill_rect(pixmap, x, y, w, h, &design.stops.first().map(|s| s.color.clone()).unwrap_or(Rgba::black()));
            return;
        }
        BgMode::LinearH => LinearGradient::new(
            Point::from_xy(x, y + h * 0.5),
            Point::from_xy(x + w, y + h * 0.5),
            stops, SpreadMode::Pad, Transform::identity(),
        ),
        BgMode::LinearV => LinearGradient::new(
            Point::from_xy(x + w * 0.5, y),
            Point::from_xy(x + w * 0.5, y + h),
            stops, SpreadMode::Pad, Transform::identity(),
        ),
        BgMode::Diagonal => LinearGradient::new(
            Point::from_xy(x, y),
            Point::from_xy(x + w, y + h),
            stops, SpreadMode::Pad, Transform::identity(),
        ),
        BgMode::Radial => RadialGradient::new(
            Point::from_xy(x + w * 0.5, y + h * 0.5),
            Point::from_xy(x + w * 0.5, y + h * 0.5),
            w.max(h) * 0.6,
            stops, SpreadMode::Pad, Transform::identity(),
        ),
        BgMode::Vignette => {
            // Vignette: draw radial with stops reversed for dark-edge effect
            let mut vstops = stops;
            vstops.reverse();
            for (i, s) in vstops.iter_mut().enumerate() {
                // remap: stop that was at 1.0 becomes 0.0 etc.
                let _ = i; // position already set in bg_stops; we need to flip
            }
            // Rebuild with flipped positions
            let vrev: Vec<GradientStop> = design.stops.iter().rev()
                .enumerate()
                .map(|(i, s)| GradientStop::new(i as f32 / (design.stops.len() - 1).max(1) as f32, to_color(&s.color)))
                .collect();
            RadialGradient::new(
                Point::from_xy(x + w * 0.5, y + h * 0.5),
                Point::from_xy(x + w * 0.5, y + h * 0.5),
                w.max(h) * 0.7,
                vrev, SpreadMode::Pad, Transform::identity(),
            )
        }
    };

    if let Some(shader) = shader {
        let mut paint = Paint::default();
        paint.shader = shader;
        paint.anti_alias = false;
        pixmap.fill_path(&PathBuilder::from_rect(rect), &paint, FillRule::Winding, Transform::identity(), None);
    }
}

/// Fill the frame background, using design if set, otherwise the solid Rgba colour.
fn fill_background(pixmap: &mut Pixmap, w: f32, h: f32, solid: &Rgba, design: &Option<BackgroundDesign>) {
    match design {
        Some(d) => fill_design(pixmap, 0.0, 0.0, w, h, d),
        None    => fill_rect(pixmap, 0.0, 0.0, w, h, solid),
    }
}

/// Fill the lower-third band, using band_design if set.
fn fill_band(pixmap: &mut Pixmap, bx: f32, by: f32, bw: f32, bh: f32, solid: &Rgba, design: &Option<BackgroundDesign>) {
    match design {
        Some(d) => fill_design(pixmap, bx, by, bw, bh, d),
        None    => fill_rect(pixmap, bx, by, bw, bh, solid),
    }
}

fn fill_rounded_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, r: f32, color: &Rgba) {
    if w <= 0.0 || h <= 0.0 { return; }
    let r = r.min(w / 2.0).min(h / 2.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(to_color(color));
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
    }
}

// ── text helpers ──────────────────────────────────────────────────────────────

fn measure<F: Font>(font: &PxScaleFont<F>, text: &str) -> f32 {
    let mut x = 0.0f32;
    let mut last = None;
    for c in text.chars() {
        let id = font.glyph_id(c);
        if let Some(prev) = last { x += font.kern(prev, id); }
        x += font.h_advance(id);
        last = Some(id);
    }
    x
}

fn wrap<F: Font>(font: &PxScaleFont<F>, text: &str, max_w: f32) -> Vec<String> {
    let mut out = Vec::new();
    for para in text.split('\n') {
        let words: Vec<&str> = para.split_whitespace().collect();
        if words.is_empty() { out.push(String::new()); continue; }
        let mut cur = String::new();
        for word in words {
            let cand = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
            if measure(font, &cand) <= max_w || cur.is_empty() { cur = cand; }
            else { out.push(cur); cur = word.to_string(); }
        }
        if !cur.is_empty() { out.push(cur); }
    }
    out
}

fn paint_line<F: Font>(
    pixmap: &mut Pixmap, font: &PxScaleFont<F>,
    text: &str, x: f32, baseline_y: f32, color: &Rgba,
) {
    let mut cx = x;
    let mut last = None;
    for c in text.chars() {
        let id = font.glyph_id(c);
        if let Some(prev) = last { cx += font.kern(prev, id); }
        let glyph = id.with_scale_and_position(font.scale(), ab_glyph::point(cx, baseline_y));
        if let Some(og) = font.outline_glyph(glyph) {
            let b = og.px_bounds();
            og.draw(|gx, gy, cov| {
                let px = b.min.x as i32 + gx as i32;
                let py = b.min.y as i32 + gy as i32;
                if px < 0 || py < 0 { return; }
                let (px, py) = (px as u32, py as u32);
                if px >= pixmap.width() || py >= pixmap.height() { return; }
                let a = (cov * color.a as f32) as u16;
                let inv = 255 - a;
                let i = (py * pixmap.width() + px) as usize * 4;
                let d = pixmap.data_mut();
                d[i]   = ((color.r as u16 * a + d[i]   as u16 * inv) / 255) as u8;
                d[i+1] = ((color.g as u16 * a + d[i+1] as u16 * inv) / 255) as u8;
                d[i+2] = ((color.b as u16 * a + d[i+2] as u16 * inv) / 255) as u8;
                d[i+3] = (a + d[i+3] as u16 * inv / 255) as u8;
            });
        }
        cx += font.h_advance(id);
        last = Some(id);
    }
}

fn paint_block<F: Font>(
    pixmap: &mut Pixmap, font: &PxScaleFont<F>,
    lines: &[String], area_x: f32, area_w: f32,
    baseline_start: f32, line_h: f32, color: &Rgba,
) {
    for (i, line) in lines.iter().enumerate() {
        let w = measure(font, line);
        let x = area_x + (area_w - w) / 2.0;
        paint_line(pixmap, font, line, x, baseline_start + i as f32 * line_h, color);
    }
}

// ── public render entry ───────────────────────────────────────────────────────

pub fn render_frame(verse_text: &str, reference: &str, cfg: &PresentConfig) -> Result<Frame, String> {
    match cfg.template {
        Template::FullScreen       => render_fullscreen(verse_text, reference, cfg),
        Template::LowerThird       => render_lower_third(verse_text, reference, cfg, false, false),
        Template::LowerThirdAccent => render_lower_third(verse_text, reference, cfg, true, false),
        Template::LowerThirdSplit  => render_lower_third(verse_text, reference, cfg, true, true),
        Template::CardCenter       => render_card(verse_text, reference, cfg),
        Template::MinimalText      => render_minimal(verse_text, reference, cfg),
    }
}

// ── template: full screen ─────────────────────────────────────────────────────

fn render_fullscreen(verse_text: &str, reference: &str, cfg: &PresentConfig) -> Result<Frame, String> {
    let (w, h) = (cfg.width, cfg.height);
    let mut pixmap = Pixmap::new(w, h).ok_or("pixmap alloc")?;
    fill_background(&mut pixmap, w as f32, h as f32, &cfg.background, &cfg.bg_design);

    let bold    = load_bold(cfg.verse_font_size)?;
    let regular = load_regular(cfg.reference_font_size)?;

    let pad    = w as f32 * cfg.padding_x;
    let area_x = pad;
    let area_w = w as f32 - pad * 2.0;

    let v_line_h = cfg.verse_font_size * cfg.line_spacing;
    let r_line_h = cfg.reference_font_size * cfg.line_spacing;
    let v_lines  = wrap(&bold, verse_text, area_w);
    let r_lines  = ref_lines(&regular, reference, area_w, cfg);

    let v_block_h = v_lines.len() as f32 * v_line_h;
    let gap       = cfg.reference_font_size * 0.6;
    let r_block_h = r_lines.len() as f32 * r_line_h;
    let total_h   = v_block_h + if r_lines.is_empty() { 0.0 } else { gap + r_block_h };

    let v_baseline = match cfg.position {
        TextPosition::Center     => h as f32 / 2.0 - total_h / 2.0 + bold.ascent(),
        TextPosition::LowerThird => h as f32 * 0.62 + bold.ascent(),
        TextPosition::UpperThird => h as f32 * 0.05 + bold.ascent(),
    };

    if !verse_text.is_empty() {
        paint_block(&mut pixmap, &bold, &v_lines, area_x, area_w, v_baseline, v_line_h, &cfg.verse_color);
    }
    if !r_lines.is_empty() {
        let r_baseline = v_baseline + v_block_h - bold.ascent() + gap + regular.ascent();
        paint_block(&mut pixmap, &regular, &r_lines, area_x, area_w, r_baseline, r_line_h, &cfg.reference_color);
    }

    to_frame(pixmap)
}

// ── template: lower third (shared for all three lower-third variants) ─────────

fn render_lower_third(
    verse_text: &str, reference: &str, cfg: &PresentConfig,
    accent: bool, split_ref: bool,
) -> Result<Frame, String> {
    let (w, h) = (cfg.width as f32, cfg.height as f32);
    let mut pixmap = Pixmap::new(cfg.width, cfg.height).ok_or("pixmap alloc")?;

    let band_h   = h * cfg.band_height;
    let band_y   = h - band_h;
    let accent_h = if accent { cfg.accent_px } else { 0.0 };

    // Background (above band) — typically chroma green or transparent
    fill_background(&mut pixmap, w, band_y, &cfg.background, &cfg.bg_design);

    // Band fill
    fill_band(&mut pixmap, 0.0, band_y, w, band_h, &cfg.band_color, &cfg.band_design);

    // Accent bar across the top of the band
    if accent {
        fill_rect(&mut pixmap, 0.0, band_y, w, accent_h, &cfg.accent_color);
    }

    let bold    = load_bold(cfg.verse_font_size)?;
    let regular = load_regular(cfg.reference_font_size)?;

    let pad    = w * cfg.padding_x;
    let area_x = pad;
    let area_w = w - pad * 2.0;

    let v_line_h = cfg.verse_font_size * cfg.line_spacing;
    let r_line_h = cfg.reference_font_size * cfg.line_spacing;
    let r_lines  = ref_lines(&regular, reference, area_w, cfg);

    if split_ref {
        // "Split" layout: reference text sits ON the accent bar; verse in the band below
        if accent && !r_lines.is_empty() {
            let r_baseline = band_y + accent_h / 2.0 + regular.ascent() / 2.0;
            paint_block(&mut pixmap, &regular, &r_lines, area_x, area_w,
                        r_baseline, r_line_h, &cfg.band_color); // dark text on accent
        }
        // Verse in band, below accent
        let v_lines  = wrap(&bold, verse_text, area_w);
        let v_block_h = v_lines.len() as f32 * v_line_h;
        let band_content_top = band_y + accent_h;
        let band_content_h   = band_h - accent_h;
        let v_baseline = band_content_top + (band_content_h - v_block_h) / 2.0 + bold.ascent();
        if !verse_text.is_empty() {
            paint_block(&mut pixmap, &bold, &v_lines, area_x, area_w,
                        v_baseline, v_line_h, &cfg.verse_color);
        }
    } else {
        // Standard: verse + reference stacked inside the band
        let v_lines  = wrap(&bold, verse_text, area_w);
        let v_block_h = v_lines.len() as f32 * v_line_h;
        let gap       = cfg.reference_font_size * 0.4;
        let r_block_h = r_lines.len() as f32 * r_line_h;
        let total_h   = v_block_h + if r_lines.is_empty() { 0.0 } else { gap + r_block_h };

        let content_top = band_y + accent_h;
        let content_h   = band_h - accent_h;
        let v_baseline  = content_top + (content_h - total_h) / 2.0 + bold.ascent();

        if !verse_text.is_empty() {
            paint_block(&mut pixmap, &bold, &v_lines, area_x, area_w,
                        v_baseline, v_line_h, &cfg.verse_color);
        }
        if !r_lines.is_empty() {
            let r_baseline = v_baseline + v_block_h - bold.ascent() + gap + regular.ascent();
            paint_block(&mut pixmap, &regular, &r_lines, area_x, area_w,
                        r_baseline, r_line_h, &cfg.reference_color);
        }
    }

    to_frame(pixmap)
}

// ── template: card centre ─────────────────────────────────────────────────────

fn render_card(verse_text: &str, reference: &str, cfg: &PresentConfig) -> Result<Frame, String> {
    let (w, h) = (cfg.width as f32, cfg.height as f32);
    let mut pixmap = Pixmap::new(cfg.width, cfg.height).ok_or("pixmap alloc")?;

    fill_background(&mut pixmap, w, h, &cfg.background, &cfg.bg_design);

    let bold    = load_bold(cfg.verse_font_size)?;
    let regular = load_regular(cfg.reference_font_size)?;

    let pad    = w * cfg.padding_x;
    let area_x = pad;
    let area_w = w - pad * 2.0;

    let v_line_h = cfg.verse_font_size * cfg.line_spacing;
    let r_line_h = cfg.reference_font_size * cfg.line_spacing;
    let v_lines  = wrap(&bold, verse_text, area_w);
    let r_lines  = ref_lines(&regular, reference, area_w, cfg);

    let v_block_h = v_lines.len() as f32 * v_line_h;
    let gap       = cfg.reference_font_size * 0.6;
    let r_block_h = r_lines.len() as f32 * r_line_h;
    let total_h   = v_block_h + if r_lines.is_empty() { 0.0 } else { gap + r_block_h };

    let inner_pad_v = cfg.verse_font_size * 0.6;
    let inner_pad_h = cfg.verse_font_size * 0.5;
    let card_w    = area_w + inner_pad_h * 2.0;
    let card_h    = total_h + inner_pad_v * 2.0;
    let card_x    = (w - card_w) / 2.0;
    let card_y    = (h - card_h) / 2.0;

    // Draw accent top bar of card
    let accent_card = Rgba { a: cfg.card_alpha, ..cfg.accent_color.clone() };
    fill_rounded_rect(&mut pixmap, card_x, card_y, card_w, cfg.accent_px * 1.5, cfg.card_radius, &accent_card);

    // Draw card body
    let card_fill = Rgba { a: cfg.card_alpha, ..cfg.band_color.clone() };
    fill_rounded_rect(&mut pixmap, card_x, card_y + cfg.accent_px * 1.5, card_w, card_h - cfg.accent_px * 1.5, cfg.card_radius, &card_fill);

    let v_baseline = card_y + cfg.accent_px * 1.5 + inner_pad_v + bold.ascent();
    if !verse_text.is_empty() {
        paint_block(&mut pixmap, &bold, &v_lines, area_x, area_w, v_baseline, v_line_h, &cfg.verse_color);
    }
    if !r_lines.is_empty() {
        let r_baseline = v_baseline + v_block_h - bold.ascent() + gap + regular.ascent();
        paint_block(&mut pixmap, &regular, &r_lines, area_x, area_w, r_baseline, r_line_h, &cfg.reference_color);
    }

    to_frame(pixmap)
}

// ── template: minimal text ────────────────────────────────────────────────────

fn render_minimal(verse_text: &str, reference: &str, cfg: &PresentConfig) -> Result<Frame, String> {
    let (w, h) = (cfg.width as f32, cfg.height as f32);
    let mut pixmap = Pixmap::new(cfg.width, cfg.height).ok_or("pixmap alloc")?;

    fill_background(&mut pixmap, w, h, &cfg.background, &cfg.bg_design);

    let bold    = load_bold(cfg.verse_font_size)?;
    let regular = load_regular(cfg.reference_font_size)?;

    let pad    = w * cfg.padding_x;
    let area_x = pad;
    let area_w = w - pad * 2.0;

    let v_line_h  = cfg.verse_font_size * cfg.line_spacing;
    let r_line_h  = cfg.reference_font_size * cfg.line_spacing;
    let v_lines   = wrap(&bold, verse_text, area_w);
    let r_lines   = ref_lines(&regular, reference, area_w, cfg);

    let v_block_h = v_lines.len() as f32 * v_line_h;
    let gap       = cfg.reference_font_size * 0.5;
    let r_block_h = r_lines.len() as f32 * r_line_h;
    let total_h   = v_block_h + if r_lines.is_empty() { 0.0 } else { gap + r_block_h };

    // Lower third area, centred vertically in bottom 30%
    let band_top   = h * (1.0 - cfg.band_height);
    let v_baseline = band_top + (h * cfg.band_height - total_h) / 2.0 + bold.ascent();

    // Draw a subtle drop-shadow by rendering text twice (dark offset, then colour)
    let shadow = Rgba::new(0, 0, 0, 120);
    if !verse_text.is_empty() {
        paint_block(&mut pixmap, &bold, &v_lines, area_x + 2.0, area_w, v_baseline + 2.0, v_line_h, &shadow);
        paint_block(&mut pixmap, &bold, &v_lines, area_x, area_w, v_baseline, v_line_h, &cfg.verse_color);
    }
    if !r_lines.is_empty() {
        let r_baseline = v_baseline + v_block_h - bold.ascent() + gap + regular.ascent();
        paint_block(&mut pixmap, &regular, &r_lines, area_x + 2.0, area_w, r_baseline + 2.0, r_line_h, &shadow);
        paint_block(&mut pixmap, &regular, &r_lines, area_x, area_w, r_baseline, r_line_h, &cfg.reference_color);
    }

    to_frame(pixmap)
}

// ── shared helpers ────────────────────────────────────────────────────────────

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

fn ref_lines<F: Font>(
    font: &PxScaleFont<F>, reference: &str, max_w: f32, cfg: &PresentConfig,
) -> Vec<String> {
    if cfg.show_reference && !reference.is_empty() {
        wrap(font, reference, max_w)
    } else {
        vec![]
    }
}

fn to_frame(pixmap: Pixmap) -> Result<Frame, String> {
    let w = pixmap.width();
    let h = pixmap.height();
    let raw: Vec<u8> = pixmap.pixels().iter().flat_map(|p| {
        let a = p.alpha();
        if a == 0 { return [0u8, 0, 0, 0]; }
        let s = 255u16;
        [
            (p.red()   as u16 * s / a as u16) as u8,
            (p.green() as u16 * s / a as u16) as u8,
            (p.blue()  as u16 * s / a as u16) as u8,
            a,
        ]
    }).collect();
    Ok(Frame { data: raw, width: w, height: h })
}

pub fn frame_to_png(frame: &Frame) -> Result<Vec<u8>, String> {
    let mut pixmap = Pixmap::new(frame.width, frame.height).ok_or("pixmap alloc")?;
    let data = pixmap.data_mut();
    for (i, px) in frame.data.chunks(4).enumerate() {
        let a = px[3];
        let idx = i * 4;
        data[idx]   = ((px[0] as u16 * a as u16) / 255) as u8;
        data[idx+1] = ((px[1] as u16 * a as u16) / 255) as u8;
        data[idx+2] = ((px[2] as u16 * a as u16) / 255) as u8;
        data[idx+3] = a;
    }
    pixmap.encode_png().map_err(|e| e.to_string())
}
