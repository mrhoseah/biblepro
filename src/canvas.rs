#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use crate::commands::{BackgroundDesign, BgMode, BgStop, Rgba};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = eval)]
    fn js_eval(code: &str) -> JsValue;
}

// ── built-in presets ──────────────────────────────────────────────────────────

pub struct Preset {
    pub name:   &'static str,
    pub design: fn() -> BackgroundDesign,
}

pub const BG_PRESETS: &[Preset] = &[
    Preset { name: "Midnight",    design: || BackgroundDesign::two_stop(BgMode::LinearV,  rgba(10,10,30),    rgba(0,0,0)) },
    Preset { name: "Deep Space",  design: || BackgroundDesign::two_stop(BgMode::Radial,   rgba(25,10,60),    rgba(0,0,10)) },
    Preset { name: "Worship Gold",design: || BackgroundDesign::two_stop(BgMode::Diagonal, rgba(60,40,0),     rgba(0,0,0)) },
    Preset { name: "Crimson",     design: || BackgroundDesign::two_stop(BgMode::LinearV,  rgba(80,0,20),     rgba(10,0,0)) },
    Preset { name: "Ocean",       design: || BackgroundDesign::two_stop(BgMode::Diagonal, rgba(0,30,80),     rgba(0,5,20)) },
    Preset { name: "Forest",      design: || BackgroundDesign::two_stop(BgMode::LinearV,  rgba(5,40,15),     rgba(0,10,0)) },
    Preset { name: "Sunset",      design: || BackgroundDesign::two_stop(BgMode::Diagonal, rgba(200,80,20),   rgba(80,10,60)) },
    Preset { name: "Vignette",    design: || BackgroundDesign { mode: BgMode::Vignette, stops: vec![BgStop{pos:0.0,color:rgba(40,35,60)},BgStop{pos:1.0,color:rgba(0,0,5)}] } },
    Preset { name: "Chroma",      design: || BackgroundDesign::solid(rgba(0,177,64)) },
    Preset { name: "White",       design: || BackgroundDesign::solid(rgba(255,255,255)) },
];

pub const BAND_PRESETS: &[Preset] = &[
    Preset { name: "Navy",        design: || BackgroundDesign::solid(rgba(15,15,40)) },
    Preset { name: "Slate",       design: || BackgroundDesign::solid(rgba(20,25,35)) },
    Preset { name: "Gold Fade",   design: || BackgroundDesign::two_stop(BgMode::LinearH, rgba(60,40,0),     rgba(10,8,0)) },
    Preset { name: "Blue Fade",   design: || BackgroundDesign::two_stop(BgMode::LinearH, rgba(10,30,80),    rgba(0,5,20)) },
    Preset { name: "Crimson Band",design: || BackgroundDesign::two_stop(BgMode::LinearH, rgba(80,0,20),     rgba(20,0,5)) },
    Preset { name: "Glass",       design: || BackgroundDesign { mode: BgMode::Vignette, stops: vec![BgStop{pos:0.0,color:rgba(60,60,80)},BgStop{pos:1.0,color:rgba(10,10,20)}] } },
];

fn rgba(r: u8, g: u8, b: u8) -> Rgba { Rgba { r, g, b, a: 255 } }

// ── JS canvas draw helper ─────────────────────────────────────────────────────

fn stops_json(stops: &[BgStop]) -> String {
    let parts: Vec<String> = stops.iter().map(|s| {
        format!(
            r#"{{"pos":{:.3},"r":{},"g":{},"b":{},"a":{}}}"#,
            s.pos, s.color.r, s.color.g, s.color.b, s.color.a
        )
    }).collect();
    format!("[{}]", parts.join(","))
}

pub fn build_draw_js(canvas_id: &str, design: &BackgroundDesign, show_lower: bool, band_height: f32) -> String {
    let mode = match &design.mode {
        BgMode::Solid    => "solid",
        BgMode::LinearH  => "linear_h",
        BgMode::LinearV  => "linear_v",
        BgMode::Diagonal => "diagonal",
        BgMode::Radial   => "radial",
        BgMode::Vignette => "vignette",
    };
    let stops = stops_json(&design.stops);
    let show_lower_str = if show_lower { "true" } else { "false" };
    let bh = band_height;

    format!(r#"(function(){{
  const canvas = document.getElementById('{canvas_id}');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  const W = canvas.width, H = canvas.height;
  ctx.clearRect(0,0,W,H);
  const stops = {stops};
  function sc(s){{ return 'rgba('+s.r+','+s.g+','+s.b+','+(s.a/255).toFixed(3)+')'; }}
  let grad;
  const mode = '{mode}';
  if (mode==='solid') {{
    ctx.fillStyle = stops[0] ? sc(stops[0]) : '#000';
    ctx.fillRect(0,0,W,H); return;
  }} else if (mode==='linear_h') {{
    grad = ctx.createLinearGradient(0,0,W,0);
  }} else if (mode==='linear_v') {{
    grad = ctx.createLinearGradient(0,0,0,H);
  }} else if (mode==='diagonal') {{
    grad = ctx.createLinearGradient(0,0,W,H);
  }} else if (mode==='radial') {{
    grad = ctx.createRadialGradient(W/2,H/2,0,W/2,H/2,Math.max(W,H)*0.6);
  }} else if (mode==='vignette') {{
    const rev = [...stops].reverse().map((s,i)=>{{...s, pos: i/(Math.max(stops.length-1,1))}});
    grad = ctx.createRadialGradient(W/2,H/2,0,W/2,H/2,Math.max(W,H)*0.7);
    rev.forEach(s=>grad.addColorStop(s.pos, sc(s)));
    ctx.fillStyle = grad;
    ctx.fillRect(0,0,W,H);
  }} else {{
    ctx.fillStyle='#000'; ctx.fillRect(0,0,W,H); return;
  }}
  if (mode!=='vignette') {{
    stops.forEach(s=>grad.addColorStop(s.pos, sc(s)));
    ctx.fillStyle = grad;
    ctx.fillRect(0,0,W,H);
  }}
  if ({show_lower_str}) {{
    const by = H*(1.0-{bh:.3});
    ctx.fillStyle='rgba(255,255,255,0.07)';
    ctx.fillRect(0,by,W,H-by);
    ctx.save();
    ctx.setLineDash([4,3]);
    ctx.strokeStyle='rgba(255,255,255,0.45)';
    ctx.lineWidth=1;
    ctx.strokeRect(0.5,by+0.5,W-1,(H-by)-1);
    ctx.restore();
    ctx.fillStyle='rgba(255,255,255,0.55)';
    ctx.font='bold 9px system-ui';
    ctx.fillText('BAND AREA',6,by+13);
  }}
}})();"#)
}

// ── canvas designer component ─────────────────────────────────────────────────

#[derive(PartialEq, Clone)]
pub enum DesignerTarget { Background, Band }

#[component]
pub fn CanvasDesigner(
    /// Current design value
    design: BackgroundDesign,
    /// Which layer this designer controls
    target: DesignerTarget,
    /// Show lower-third band overlay in preview
    band_height: f32,
    /// Called when the user clicks "Apply"
    on_apply: EventHandler<BackgroundDesign>,
    /// Called when the user clicks "Clear" (resets to solid)
    on_clear: EventHandler<()>,
) -> Element {
    let canvas_id = match target {
        DesignerTarget::Background => "cd-canvas-bg",
        DesignerTarget::Band       => "cd-canvas-band",
    };

    let mut mode      = use_signal(|| design.mode.clone());
    let mut stops     = use_signal(|| design.stops.clone());
    let mut show_band = use_signal(|| target == DesignerTarget::Background);
    let mut active_stop = use_signal(|| 0usize);

    // Sync from prop when parent changes (e.g. preset applied)
    {
        let d = design.clone();
        use_effect(move || {
            mode.set(d.mode.clone());
            stops.set(d.stops.clone());
        });
    }

    // Redraw canvas whenever state changes
    {
        let cid = canvas_id;
        let bh = band_height;
        use_effect(move || {
            let design = BackgroundDesign { mode: mode(), stops: stops() };
            let js = build_draw_js(cid, &design, show_band(), bh);
            js_eval(&js);
        });
    }

    let current_design = move || BackgroundDesign { mode: mode(), stops: stops() };

    // Stop color editor
    let stop_count = stops.read().len();

    rsx! {
        div { class: "canvas-designer",

            // ── preview canvas ────────────────────────────────────────────
            div { class: "cd-canvas-wrap",
                canvas { id: "{canvas_id}", class: "cd-canvas", width: "320", height: "180" }
                div { class: "cd-canvas-overlay",
                    span { class: "cd-canvas-label",
                        match target {
                            DesignerTarget::Background => "Background Preview",
                            DesignerTarget::Band       => "Band Preview",
                        }
                    }
                    if target == DesignerTarget::Background {
                        button {
                            class: if show_band() { "cd-view-btn active" } else { "cd-view-btn" },
                            onclick: move |_| show_band.set(!show_band()),
                            "Show band"
                        }
                    }
                }
            }

            // ── mode selector ─────────────────────────────────────────────
            div { class: "cd-section",
                span { class: "cd-label", "Fill Mode" }
                div { class: "cd-mode-row",
                    for m in BgMode::all() {
                        {
                            let mv = m.clone();
                            let is_active = mode() == *m;
                            rsx! {
                                button {
                                    class: if is_active { "cd-mode-btn active" } else { "cd-mode-btn" },
                                    onclick: move |_| mode.set(mv.clone()),
                                    "{m.label()}"
                                }
                            }
                        }
                    }
                }
            }

            // ── colour stops ──────────────────────────────────────────────
            div { class: "cd-section",
                div { class: "cd-stops-header",
                    span { class: "cd-label", "Colour Stops" }
                    if stop_count < 4 {
                        button {
                            class: "cd-stop-add",
                            onclick: move |_| {
                                let mut s = stops.write();
                                let new_pos = 1.0f32.min(s.last().map(|l| (l.pos + 1.0) / 2.0).unwrap_or(1.0));
                                s.push(BgStop { pos: new_pos, color: Rgba::black() });
                                active_stop.set(s.len() - 1);
                            },
                            "+ Add Stop"
                        }
                    }
                }

                // Stop tabs
                div { class: "cd-stop-tabs",
                    for i in 0..stop_count {
                        {
                            let bg = stops.read().get(i).map(|s| s.color.to_hex()).unwrap_or_default();
                            let is_active = active_stop() == i;
                            rsx! {
                                button {
                                    class: if is_active { "cd-stop-tab active" } else { "cd-stop-tab" },
                                    onclick: move |_| active_stop.set(i),
                                    div { class: "cd-stop-swatch", style: "background:{bg};" }
                                    span { "Stop {i+1}" }
                                    if stop_count > 2 {
                                        span {
                                            class: "cd-stop-remove",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                let mut s = stops.write();
                                                if i < s.len() { s.remove(i); }
                                                if active_stop() >= s.len() && !s.is_empty() {
                                                    active_stop.set(s.len() - 1);
                                                }
                                            },
                                            "✕"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Active stop editor
                if let Some(stop) = stops.read().get(active_stop()).cloned() {
                    div { class: "cd-stop-editor",
                        // Colour picker
                        div { class: "cd-row",
                            label { class: "cd-row-label", "Colour" }
                            input {
                                r#type: "color",
                                class: "cd-color-input",
                                value: "{stop.color.to_hex()}",
                                oninput: move |e| {
                                    if let Some(rgb) = hex_to_rgb(&e.value()) {
                                        let mut s = stops.write();
                                        if let Some(stop) = s.get_mut(active_stop()) {
                                            stop.color = rgb;
                                        }
                                    }
                                },
                            }
                        }
                        // Position slider (hidden for mode Solid and first stop)
                        if mode() != BgMode::Solid && active_stop() > 0 {
                            div { class: "cd-row",
                                label { class: "cd-row-label", "Position  {(stop.pos * 100.0) as i32}%" }
                                input {
                                    r#type: "range",
                                    class: "cd-slider",
                                    min: "0", max: "100", step: "1",
                                    value: "{(stop.pos * 100.0) as i32}",
                                    oninput: move |e| {
                                        if let Ok(v) = e.value().parse::<f32>() {
                                            let mut s = stops.write();
                                            if let Some(stop) = s.get_mut(active_stop()) {
                                                stop.pos = v / 100.0;
                                            }
                                        }
                                    },
                                }
                            }
                        }
                        // Opacity slider
                        div { class: "cd-row",
                            label { class: "cd-row-label", "Opacity  {(stop.color.a as f32/255.0*100.0) as i32}%" }
                            input {
                                r#type: "range",
                                class: "cd-slider",
                                min: "0", max: "255", step: "1",
                                value: "{stop.color.a}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<u8>() {
                                        let mut s = stops.write();
                                        if let Some(stop) = s.get_mut(active_stop()) {
                                            stop.color.a = v;
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
            }

            // ── presets ───────────────────────────────────────────────────
            div { class: "cd-section",
                span { class: "cd-label", "Presets" }
                div { class: "cd-presets",
                    for preset in match target {
                        DesignerTarget::Background => BG_PRESETS,
                        DesignerTarget::Band       => BAND_PRESETS,
                    } {
                        button {
                            class: "cd-preset-btn",
                            onclick: move |_| {
                                let d = (preset.design)();
                                mode.set(d.mode.clone());
                                stops.set(d.stops.clone());
                            },
                            "{preset.name}"
                        }
                    }
                }
            }

            // ── action buttons ────────────────────────────────────────────
            div { class: "cd-actions",
                button {
                    class: "btn-primary",
                    onclick: move |_| on_apply.call(current_design()),
                    "Apply"
                }
                button {
                    class: "btn-ghost sm-btn",
                    onclick: move |_| on_clear.call(()),
                    "Clear"
                }
            }
        }
    }
}

fn hex_to_rgb(hex: &str) -> Option<Rgba> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 { return None; }
    Some(Rgba {
        r: u8::from_str_radix(&h[0..2], 16).ok()?,
        g: u8::from_str_radix(&h[2..4], 16).ok()?,
        b: u8::from_str_radix(&h[4..6], 16).ok()?,
        a: 255,
    })
}
