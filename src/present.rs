#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use crate::commands::{
    cmd_get_books, cmd_get_chapter, cmd_get_present_config, cmd_get_translations,
    cmd_ndi_preview, cmd_push_to_all, cmd_clear_all,
    cmd_search_by_reference, cmd_set_present_config, cmd_pick_background_image,
    Book, ChapterInfo, PresentConfig, Rgba, Template, TextPosition, Translation,
};
use crate::canvas::{CanvasDesigner, DesignerTarget};
use crate::output::OutputPanel;
use crate::plans::{PremiumBadge, PremiumTeaser};

// ── helpers ───────────────────────────────────────────────────────────────────

fn hex_to_rgba(hex: &str) -> Option<Rgba> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 { return None; }
    Some(Rgba {
        r: u8::from_str_radix(&h[0..2], 16).ok()?,
        g: u8::from_str_radix(&h[2..4], 16).ok()?,
        b: u8::from_str_radix(&h[4..6], 16).ok()?,
        a: 255,
    })
}

fn rgba_to_hex(c: &Rgba) -> String { format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b) }
fn rgba_to_css(c: &Rgba) -> String { format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a as f32 / 255.0) }

#[wasm_bindgen(inline_js = "
export function active_tag() {
    return (document.activeElement && document.activeElement.tagName) || '';
}
")]
extern "C" { fn active_tag() -> String; }

fn input_focused() -> bool {
    let tag = active_tag().to_uppercase();
    matches!(tag.as_str(), "INPUT" | "TEXTAREA" | "SELECT")
}

// ── data types ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
struct QueueItem {
    text:      String,
    reference: String,
    book_id:   i32,
    chapter:   i32,
    verse:     i32,
    book_name: String,
}

// ── main view ─────────────────────────────────────────────────────────────────

#[component]
pub fn PresentView() -> Element {
    // Config & status
    let mut cfg     = use_signal(PresentConfig::default);
    let mut status  = use_signal(|| String::new());
    let mut loading = use_signal(|| false);

    // Setlist queue
    let mut queue:     Signal<Vec<QueueItem>> = use_signal(Vec::new);
    let mut queue_idx: Signal<usize>          = use_signal(|| 0);

    // LIVE — currently on screen
    let mut verse_text  = use_signal(|| String::new());
    let mut reference   = use_signal(|| String::new());
    let mut preview_src = use_signal(|| String::new());

    // STAGED — keyboard cursor, ready to push
    let mut staged_verse: Signal<i32>    = use_signal(|| 0i32);
    let mut staged_text:  Signal<String> = use_signal(String::new);
    let mut staged_ref:   Signal<String> = use_signal(String::new);

    // Translations
    let mut translations = use_signal(|| Vec::<Translation>::new());
    let mut live_trans   = use_signal(|| String::new());

    // Scripture browser
    let mut browse_book_id: Signal<i32>               = use_signal(|| 1i32);
    let mut browse_chapter: Signal<i32>               = use_signal(|| 1i32);
    let mut browse_data:    Signal<Option<ChapterInfo>> = use_signal(|| None);
    let mut browse_total:   Signal<i32>               = use_signal(|| 0i32);

    // Left panel quick-add
    let mut search_query = use_signal(|| String::new());
    let mut search_msg   = use_signal(|| String::new());
    let mut suggestions: Signal<Vec<String>> = use_signal(Vec::new);
    let mut sug_idx:     Signal<i32>         = use_signal(|| -1i32);
    let mut show_sugs:   Signal<bool>        = use_signal(|| false);

    // Center Jump Bar
    let mut jump_query      = use_signal(|| String::new());
    let mut jump_msg        = use_signal(|| String::new());
    let mut jump_sugs:      Signal<Vec<String>> = use_signal(Vec::new);
    let mut jump_sug_idx:   Signal<i32>         = use_signal(|| -1i32);
    let mut show_jump_sugs: Signal<bool>        = use_signal(|| false);

    // Session metadata
    let mut session_open   = use_signal(|| false);
    let mut sermon_title   = use_signal(|| String::new());
    let mut sermon_series  = use_signal(|| String::new());
    let mut sermon_speaker = use_signal(|| String::new());

    // History
    let mut history: Signal<Vec<QueueItem>> = use_signal(Vec::new);

    // Books
    let mut books: Signal<Vec<Book>> = use_signal(Vec::new);

    // UI state
    let mut cfg_open:         Signal<bool> = use_signal(|| false);
    let mut preview_size:     Signal<u8>   = use_signal(|| 1u8);
    let mut left_tab:         Signal<u8>   = use_signal(|| 0u8);
    let mut cfg_tab:          Signal<u8>   = use_signal(|| 0u8);
    let mut bg_tab:           Signal<u8>   = use_signal(|| 0u8);
    let mut bg_image_picking: Signal<bool> = use_signal(|| false);

    // ── Effects ───────────────────────────────────────────────────────────────

    use_effect(move || {
        spawn(async move {
            cfg.set(cmd_get_present_config().await);
            let ts = cmd_get_translations().await;
            if let Some(first) = ts.first() { live_trans.set(first.id.clone()); }
            translations.set(ts);
            books.set(cmd_get_books().await);
        });
    });

    use_effect(move || {
        let t = live_trans(); let b = browse_book_id(); let c = browse_chapter();
        if t.is_empty() || b == 0 { return; }
        spawn(async move {
            let info = cmd_get_chapter(&t, b, c).await;
            if let Some(ref d) = info { browse_total.set(d.total_chapters); }
            browse_data.set(info);
        });
    });

    use_effect(move || {
        let vt = verse_text(); let rf = reference();
        if vt.trim().is_empty() { return; }
        let v = vt.clone(); let r = rf.clone();
        spawn(async move {
            if let Ok(p) = cmd_ndi_preview(&v, &r).await {
                preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
            }
        });
    });

    use_effect(move || {
        let t = live_trans(); let b = browse_book_id(); let c = browse_chapter();
        let curr_v = staged_verse();
        if t.is_empty() || b == 0 || c == 0 || curr_v == 0 { return; }
        spawn(async move {
            if let Some(info) = cmd_get_chapter(&t, b, c).await {
                if let Some(v) = info.verses.iter().find(|v| v.verse == curr_v) {
                    let new_text = v.text.clone();
                    let new_ref  = format!("{} {}:{}", info.book_name, c, v.verse);
                    verse_text.set(new_text.clone()); reference.set(new_ref.clone());
                    spawn(async move {
                        if let Ok(p) = cmd_push_to_all(&new_text, &new_ref).await {
                            preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                            status.set("Live — version switched.".to_string());
                        }
                    });
                }
            }
        });
    });

    use_effect(move || {
        let q = search_query().trim().to_lowercase();
        if q.len() < 2 { suggestions.set(Vec::new()); show_sugs.set(false); return; }
        let sug: Vec<String> = books().iter()
            .filter(|b| b.name.to_lowercase().starts_with(&q) || b.short_name.to_lowercase().starts_with(&q))
            .take(6).map(|b| b.name.clone()).collect();
        show_sugs.set(!sug.is_empty());
        suggestions.set(sug); sug_idx.set(-1);
    });

    use_effect(move || {
        let q = jump_query().trim().to_lowercase();
        if q.len() < 2 { jump_sugs.set(Vec::new()); show_jump_sugs.set(false); return; }
        let sug: Vec<String> = books().iter()
            .filter(|b| b.name.to_lowercase().starts_with(&q) || b.short_name.to_lowercase().starts_with(&q))
            .take(8).map(|b| b.name.clone()).collect();
        show_jump_sugs.set(!sug.is_empty());
        jump_sugs.set(sug); jump_sug_idx.set(-1);
    });

    // ── Push helpers ──────────────────────────────────────────────────────────

    let do_push = move |_| {
        let v = verse_text(); let r = reference();
        if v.trim().is_empty() { return; }
        let v2 = v.clone(); let r2 = r.clone();
        spawn(async move {
            loading.set(true);
            match cmd_push_to_all(&v, &r).await {
                Ok(p) => {
                    preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                    status.set("Live.".to_string());
                    let mut h = history.write();
                    h.retain(|i| i.reference != r2);
                    h.insert(0, QueueItem { text: v2, reference: r2, book_id: 0, chapter: 0, verse: 0, book_name: String::new() });
                    if h.len() > 20 { h.truncate(20); }
                }
                Err(e) => status.set(format!("Error: {e}")),
            }
            loading.set(false);
        });
    };

    let do_push_next = move |_| {
        let v = verse_text(); let r = reference();
        if v.trim().is_empty() { return; }
        let v2 = v.clone(); let r2 = r.clone();
        let len = queue.read().len(); let idx = queue_idx();
        spawn(async move {
            loading.set(true);
            match cmd_push_to_all(&v, &r).await {
                Ok(p) => {
                    preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                    status.set("Live.".to_string());
                    let mut h = history.write();
                    h.retain(|i| i.reference != r2);
                    h.insert(0, QueueItem { text: v2, reference: r2, book_id: 0, chapter: 0, verse: 0, book_name: String::new() });
                    if h.len() > 20 { h.truncate(20); }
                }
                Err(e) => { status.set(format!("Error: {e}")); loading.set(false); return; }
            }
            if idx + 1 < len {
                let next = idx + 1;
                queue_idx.set(next);
                if let Some(item) = queue.read().get(next) {
                    verse_text.set(item.text.clone()); reference.set(item.reference.clone());
                    browse_book_id.set(item.book_id); browse_chapter.set(item.chapter);
                    staged_verse.set(item.verse);
                    staged_text.set(item.text.clone()); staged_ref.set(item.reference.clone());
                }
            }
            loading.set(false);
        });
    };

    let do_clear = move |_| {
        spawn(async move {
            if let Ok(p) = cmd_clear_all().await {
                preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                verse_text.set(String::new()); reference.set(String::new());
                status.set("Cleared.".to_string());
            }
        });
    };

    // Click-to-live: stage + push immediately
    let push_verse = move |text: String, rref: String, book_id: i32, chapter: i32, verse_num: i32| {
        verse_text.set(text.clone()); reference.set(rref.clone());
        staged_verse.set(verse_num); staged_text.set(text.clone()); staged_ref.set(rref.clone());
        let t2 = text.clone(); let r2 = rref.clone();
        spawn(async move {
            loading.set(true);
            match cmd_push_to_all(&t2, &r2).await {
                Ok(p) => {
                    preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                    status.set("Live.".to_string());
                    let mut h = history.write();
                    h.retain(|i| i.reference != r2);
                    h.insert(0, QueueItem { text: t2, reference: r2, book_id, chapter, verse: verse_num, book_name: String::new() });
                    if h.len() > 20 { h.truncate(20); }
                }
                Err(e) => status.set(format!("Error: {e}")),
            }
            loading.set(false);
        });
    };

    // Right-click: stage only, no push
    let stage_only = move |text: String, rref: String, verse_num: i32| {
        staged_verse.set(verse_num); staged_text.set(text); staged_ref.set(rref);
    };

    // Setlist loader
    let mut load_queue_item = move |i: usize| {
        queue_idx.set(i);
        if let Some(item) = queue.read().get(i) {
            verse_text.set(item.text.clone()); reference.set(item.reference.clone());
            browse_book_id.set(item.book_id); browse_chapter.set(item.chapter);
            staged_verse.set(item.verse);
            staged_text.set(item.text.clone()); staged_ref.set(item.reference.clone());
        }
    };

    // Arrow-key staging — updates STAGED card without pushing
    let stage_prev_verse = move || {
        if let Some(ctx) = browse_data() {
            let curr = staged_verse();
            let curr = if curr == 0 { ctx.verses.first().map(|v| v.verse).unwrap_or(1) } else { curr };
            if let Some(pos) = ctx.verses.iter().position(|v| v.verse == curr) {
                if pos > 0 {
                    let v = &ctx.verses[pos - 1];
                    staged_verse.set(v.verse);
                    staged_text.set(v.text.clone());
                    staged_ref.set(format!("{} {}:{}", ctx.book_name, ctx.chapter, v.verse));
                }
            }
        }
    };

    let stage_next_verse = move || {
        if let Some(ctx) = browse_data() {
            let curr = staged_verse();
            let start_pos = ctx.verses.iter().position(|v| v.verse == curr).unwrap_or(0);
            let next_pos = if curr == 0 { 0 } else { start_pos + 1 };
            if next_pos < ctx.verses.len() {
                let v = &ctx.verses[next_pos];
                staged_verse.set(v.verse);
                staged_text.set(v.text.clone());
                staged_ref.set(format!("{} {}:{}", ctx.book_name, ctx.chapter, v.verse));
            }
        }
    };

    // Left panel quick-add
    let do_add = move || {
        let q = search_query().trim().to_string(); let t = live_trans();
        if q.is_empty() || t.is_empty() { return; }
        spawn(async move {
            search_msg.set("Searching…".to_string());
            match cmd_search_by_reference(&t, &q).await {
                Some(v) => {
                    let ref_ = format!("{} {}:{}", v.book_name, v.chapter, v.verse);
                    verse_text.set(v.text.clone()); reference.set(ref_.clone());
                    staged_text.set(v.text.clone()); staged_ref.set(ref_.clone());
                    browse_book_id.set(v.book_id); browse_chapter.set(v.chapter);
                    staged_verse.set(v.verse);
                    let mut qw = queue.write();
                    qw.push(QueueItem { text: v.text, reference: ref_.clone(), book_id: v.book_id, chapter: v.chapter, verse: v.verse, book_name: v.book_name });
                    queue_idx.set(qw.len() - 1); drop(qw);
                    search_query.set(String::new());
                    search_msg.set(format!("✓ {ref_}"));
                }
                None => search_msg.set("Not found.".to_string()),
            }
        });
    };
    let do_add2 = do_add.clone();

    // Jump bar — instant live push
    let do_jump = move || {
        let q = jump_query().trim().to_string(); let t = live_trans();
        if q.is_empty() || t.is_empty() { return; }
        spawn(async move {
            jump_msg.set("…".to_string()); show_jump_sugs.set(false);
            match cmd_search_by_reference(&t, &q).await {
                Some(v) => {
                    let ref_ = format!("{} {}:{}", v.book_name, v.chapter, v.verse);
                    browse_book_id.set(v.book_id); browse_chapter.set(v.chapter);
                    staged_verse.set(v.verse); staged_text.set(v.text.clone()); staged_ref.set(ref_.clone());
                    verse_text.set(v.text.clone()); reference.set(ref_.clone());
                    let t2 = v.text.clone(); let r2 = ref_.clone();
                    spawn(async move {
                        if let Ok(p) = cmd_push_to_all(&t2, &r2).await {
                            preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                            status.set("Live.".to_string());
                            let mut h = history.write();
                            h.retain(|i| i.reference != r2);
                            h.insert(0, QueueItem { text: t2, reference: r2, book_id: 0, chapter: 0, verse: 0, book_name: String::new() });
                            if h.len() > 20 { h.truncate(20); }
                        }
                    });
                    jump_query.set(String::new()); jump_msg.set(String::new());
                }
                None => jump_msg.set("Not found.".to_string()),
            }
        });
    };
    let do_jump2 = do_jump.clone();

    // ── Precompute for RSX ────────────────────────────────────────────────────
    let q_len    = queue.read().len();
    let q_idx    = queue_idx();
    let can_push = !loading() && !verse_text().trim().is_empty();
    let has_staged = !staged_text().trim().is_empty() && staged_ref() != reference();

    let queue_snap: Vec<(usize, String, String)> = queue.read().iter().enumerate()
        .map(|(i, it)| (i, it.reference.clone(), it.text.chars().take(52).collect()))
        .collect();

    let up_next: Option<(String, String)> = queue.read().get(q_idx + 1)
        .map(|it| (it.reference.clone(), it.text.chars().take(80).collect()));

    let hist_snap: Vec<(String, String, String)> = history.read().iter()
        .map(|h| (h.reference.clone(), h.text.chars().take(55).collect::<String>(), h.text.clone()))
        .collect();

    let bg          = cfg().background.clone();
    let sel_black   = if bg.r==0   && bg.g==0   && bg.b==0   { "preset-btn selected" } else { "preset-btn" };
    let sel_green   = if bg.r==0   && bg.g==177 && bg.b==64  { "preset-btn selected" } else { "preset-btn" };
    let sel_blue    = if bg.r==0   && bg.g==0   && bg.b==255 { "preset-btn selected" } else { "preset-btn" };
    let sel_white   = if bg.r==255 && bg.g==255 && bg.b==255 { "preset-btn selected" } else { "preset-btn" };
    let sel_dblue   = if bg.r==10  && bg.g==15  && bg.b==60  { "preset-btn selected" } else { "preset-btn" };
    let sel_purple  = if bg.r==30  && bg.g==0   && bg.b==60  { "preset-btn selected" } else { "preset-btn" };

    rsx! {
        div {
            class: "present-root",
            tabindex: "0",

            // ══ LEFT: Setlist & session ══════════════════════════════════════
            div { class: "present-left",

                div {
                    class: if !verse_text().trim().is_empty() { "present-live-bar live" } else { "present-live-bar" },
                    div { class: "plb-dot" }
                    if !reference().is_empty() {
                        span { class: "plb-ref", "{reference()}" }
                    }
                    span { class: "plb-status",
                        if !verse_text().trim().is_empty() { "LIVE" } else { "STANDBY" }
                    }
                }

                OutputPanel {}

                div { class: "pres-trans-bar",
                    span { class: "pres-trans-label", "Version" }
                    div { class: "pres-trans-chips",
                        for t in translations() {
                            {
                                let tid = t.id.clone();
                                let is_active = live_trans() == t.id;
                                rsx! {
                                    button {
                                        class: if is_active { "pres-trans-chip active" } else { "pres-trans-chip" },
                                        title: "{t.name}  ·  {t.language}",
                                        onclick: move |_| live_trans.set(tid.clone()),
                                        "{t.abbreviation}"
                                    }
                                }
                            }
                        }
                        if translations().is_empty() {
                            span { class: "pres-trans-empty", "Import a translation in Library" }
                        }
                    }
                }

                div { class: "panel-section sermon-section",
                    button {
                        class: if session_open() { "section-toggle active" } else { "section-toggle" },
                        onclick: move |_| session_open.set(!session_open()),
                        span { class: "toggle-arrow", if session_open() { "▲" } else { "▼" } }
                        "Session"
                        if !sermon_title().is_empty() {
                            span { class: "sermon-badge", "{sermon_title()}" }
                        }
                    }
                    if session_open() {
                        div { class: "sermon-fields",
                            div { class: "sermon-field",
                                label { class: "sf-label", "Title" }
                                input { class: "search-input", placeholder: "Sermon title…", value: "{sermon_title}", oninput: move |e| sermon_title.set(e.value()) }
                            }
                            div { class: "sermon-field",
                                label { class: "sf-label", "Series" }
                                input { class: "search-input", placeholder: "Series name…", value: "{sermon_series}", oninput: move |e| sermon_series.set(e.value()) }
                            }
                            div { class: "sermon-field",
                                label { class: "sf-label", "Speaker" }
                                input { class: "search-input", placeholder: "Speaker name…", value: "{sermon_speaker}", oninput: move |e| sermon_speaker.set(e.value()) }
                            }
                        }
                    }
                }

                div { class: "panel-section setlist-section",
                    div { class: "left-tab-bar",
                        button {
                            class: if left_tab() == 0 { "ltab active" } else { "ltab" },
                            onclick: move |_| left_tab.set(0),
                            "Setlist"
                            if q_len > 0 { span { class: "ltab-badge", "{q_len}" } }
                        }
                        button {
                            class: if left_tab() == 1 { "ltab active" } else { "ltab" },
                            onclick: move |_| left_tab.set(1),
                            "History"
                            if !hist_snap.is_empty() { span { class: "ltab-badge", "{hist_snap.len()}" } }
                        }
                    }

                    if left_tab() == 1 {
                        div { class: "history-panel",
                            if hist_snap.is_empty() {
                                div { class: "history-empty",
                                    div { class: "history-empty-icon", "◷" }
                                    p { "No history yet." }
                                    p { class: "history-empty-hint", "Every pushed verse appears here." }
                                }
                            } else {
                                div { class: "history-list",
                                    for (ref_, preview, full_text) in hist_snap.clone() {
                                        {
                                            let r_push = ref_.clone(); let t_push = full_text.clone();
                                            rsx! {
                                                div { class: "hist-item",
                                                    div { class: "hist-body",
                                                        div { class: "hist-ref", "{ref_}" }
                                                        div { class: "hist-preview", "{preview}" }
                                                    }
                                                    button {
                                                        class: "hist-push-btn", title: "Push live again",
                                                        onclick: move |_| {
                                                            let v = t_push.clone(); let r = r_push.clone();
                                                            verse_text.set(v.clone()); reference.set(r.clone());
                                                            spawn(async move {
                                                                if let Ok(p) = cmd_push_to_all(&v, &r).await {
                                                                    preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                                                                    status.set("Re-pushed.".to_string());
                                                                }
                                                            });
                                                        },
                                                        "▶"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if left_tab() == 0 {
                        div { class: "quick-add-wrap",
                            div { class: "quick-add-row",
                                input {
                                    class: "search-input",
                                    placeholder: "John 3:16 — add to setlist",
                                    value: "{search_query}",
                                    oninput: move |e| { search_query.set(e.value()); search_msg.set(String::new()); },
                                    onblur: move |_| show_sugs.set(false),
                                    onkeydown: move |e| {
                                        match e.key() {
                                            Key::ArrowDown => { let max = suggestions().len() as i32; if max > 0 { sug_idx.set((sug_idx() + 1).min(max - 1)); } }
                                            Key::ArrowUp   => { sug_idx.set((sug_idx() - 1).max(-1)); }
                                            Key::Enter     => {
                                                let idx = sug_idx();
                                                if idx >= 0 {
                                                    if let Some(s) = suggestions().get(idx as usize).cloned() { search_query.set(format!("{s} ")); show_sugs.set(false); sug_idx.set(-1); }
                                                } else { do_add2(); }
                                            }
                                            Key::Escape => show_sugs.set(false),
                                            _ => {}
                                        }
                                    },
                                }
                                button { class: "btn-add", title: "Add to setlist", onclick: move |_| do_add(), "+" }
                            }
                            if show_sugs() && !suggestions().is_empty() {
                                div { class: "ac-dropdown",
                                    for (i, s) in suggestions().into_iter().enumerate() {
                                        {
                                            let sc = s.clone();
                                            rsx! {
                                                div {
                                                    key: "{i}",
                                                    class: if sug_idx() == i as i32 { "ac-item ac-selected" } else { "ac-item" },
                                                    onmousedown: move |_| { search_query.set(format!("{sc} ")); show_sugs.set(false); sug_idx.set(-1); },
                                                    "{s}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if !search_msg().is_empty() { div { class: "search-msg", "{search_msg}" } }

                        div { class: "queue-list",
                            if queue_snap.is_empty() {
                                div { class: "queue-empty", "Type a reference and press " strong { "+" } " or Enter." }
                            }
                            for (i, ref_, preview) in queue_snap {
                                div {
                                    key: "{i}",
                                    class: if i == q_idx { "queue-item active" } else { "queue-item" },
                                    onclick: move |_| load_queue_item(i),
                                    div { class: "qi-left", div { class: "qi-num", "{i + 1}" } }
                                    div { class: "qi-body",
                                        div { class: "qi-ref", "{ref_}" }
                                        div { class: "qi-preview", "{preview}…" }
                                    }
                                    button {
                                        class: "qi-remove", title: "Remove",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            let mut q = queue.write();
                                            if i < q.len() { q.remove(i); }
                                            let curr = queue_idx();
                                            if curr >= q.len() && curr > 0 { queue_idx.set(curr - 1); }
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }

                        div { class: "queue-nav",
                            button { class: "queue-nav-btn", disabled: q_idx == 0, onclick: move |_| { let i = queue_idx(); if i > 0 { load_queue_item(i - 1); } }, "◀ Prev" }
                            span { class: "queue-pos", if q_len > 0 { "{q_idx + 1} / {q_len}" } else { "—" } }
                            button { class: "queue-nav-btn", disabled: q_idx + 1 >= q_len, onclick: move |_| { let i = queue_idx(); let len = queue.read().len(); if i + 1 < len { load_queue_item(i + 1); } }, "Next ▶" }
                        }
                    }
                }
            } // end present-left

            // ══ CENTER: Scripture Theater ════════════════════════════════════
            div {
                class: "present-center",
                tabindex: "0",

                onkeydown: move |e: KeyboardEvent| {
                    if input_focused() { return; }
                    match e.key() {
                        Key::Character(ref s) if s == " " => {
                            e.prevent_default();
                            let st = staged_text(); let sr = staged_ref();
                            if st.trim().is_empty() { return; }
                            verse_text.set(st.clone()); reference.set(sr.clone());
                            let t2 = st.clone(); let r2 = sr.clone();
                            spawn(async move {
                                loading.set(true);
                                match cmd_push_to_all(&t2, &r2).await {
                                    Ok(p) => {
                                        preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                                        status.set("Live.".to_string());
                                        let mut h = history.write();
                                        h.retain(|i| i.reference != r2);
                                        h.insert(0, QueueItem { text: t2, reference: r2, book_id: 0, chapter: 0, verse: 0, book_name: String::new() });
                                        if h.len() > 20 { h.truncate(20); }
                                    }
                                    Err(err) => status.set(format!("Error: {err}")),
                                }
                                loading.set(false);
                            });
                        }
                        Key::F5 => { e.prevent_default(); }
                        Key::ArrowDown | Key::ArrowRight => { e.prevent_default(); stage_next_verse(); }
                        Key::ArrowUp   | Key::ArrowLeft  => { e.prevent_default(); stage_prev_verse(); }
                        Key::Character(ref s) if s == "]" => { let c = browse_chapter(); let max = browse_total(); if c < max { browse_chapter.set(c + 1); staged_verse.set(0); staged_text.set(String::new()); staged_ref.set(String::new()); } }
                        Key::Character(ref s) if s == "[" => { let c = browse_chapter(); if c > 1 { browse_chapter.set(c - 1); staged_verse.set(0); staged_text.set(String::new()); staged_ref.set(String::new()); } }
                        Key::Character(ref s) if s == "." => { let i = queue_idx(); let len = queue.read().len(); if i + 1 < len { load_queue_item(i + 1); } }
                        Key::Character(ref s) if s == "," => { let i = queue_idx(); if i > 0 { load_queue_item(i - 1); } }
                        Key::Character(ref s) if s == "c" || s == "C" => {
                            spawn(async move {
                                if let Ok(p) = cmd_clear_all().await {
                                    preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                                    verse_text.set(String::new()); reference.set(String::new());
                                    status.set("Cleared.".to_string());
                                }
                            });
                        }
                        _ => {}
                    }
                },

                // ── Jump Bar — instant live push ──────────────────────────────
                div { class: "jump-bar-wrap",
                    div { class: "jump-bar-row",
                        span { class: "jump-bar-icon", "⚡" }
                        div { class: "jump-bar-field",
                            input {
                                class: "jump-bar-input",
                                placeholder: "Jump live: John 3:16 — Enter to push",
                                value: "{jump_query}",
                                oninput: move |e| { jump_query.set(e.value()); jump_msg.set(String::new()); },
                                onblur: move |_| show_jump_sugs.set(false),
                                onkeydown: move |e| {
                                    match e.key() {
                                        Key::ArrowDown => { let max = jump_sugs().len() as i32; if max > 0 { jump_sug_idx.set((jump_sug_idx() + 1).min(max - 1)); } }
                                        Key::ArrowUp   => { jump_sug_idx.set((jump_sug_idx() - 1).max(-1)); }
                                        Key::Enter     => {
                                            let idx = jump_sug_idx();
                                            if idx >= 0 {
                                                if let Some(s) = jump_sugs().get(idx as usize).cloned() { jump_query.set(format!("{s} ")); show_jump_sugs.set(false); jump_sug_idx.set(-1); }
                                            } else { do_jump(); }
                                        }
                                        Key::Escape => { show_jump_sugs.set(false); jump_query.set(String::new()); jump_msg.set(String::new()); }
                                        _ => {}
                                    }
                                },
                            }
                            if !jump_msg().is_empty() { span { class: "jump-bar-msg", "{jump_msg}" } }
                        }
                        button { class: "jump-bar-go", onclick: move |_| do_jump2(), "▶ Live" }
                    }
                    if show_jump_sugs() && !jump_sugs().is_empty() {
                        div { class: "jump-ac-dropdown",
                            for (i, s) in jump_sugs().into_iter().enumerate() {
                                {
                                    let sc = s.clone();
                                    rsx! {
                                        div {
                                            key: "{i}",
                                            class: if jump_sug_idx() == i as i32 { "jump-ac-item selected" } else { "jump-ac-item" },
                                            onmousedown: move |_| { jump_query.set(format!("{sc} ")); show_jump_sugs.set(false); jump_sug_idx.set(-1); },
                                            "{s}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Book + chapter navigator ──────────────────────────────────
                div { class: "browse-nav-bar",
                    select {
                        class: "browse-book-select",
                        value: "{browse_book_id}",
                        onchange: move |e| {
                            if let Ok(id) = e.value().parse::<i32>() {
                                browse_book_id.set(id); browse_chapter.set(1);
                                staged_verse.set(0); staged_text.set(String::new()); staged_ref.set(String::new());
                            }
                        },
                        option { value: "0", disabled: true, "— Book —" }
                        {
                            let bks = books();
                            let ot: Vec<_> = bks.iter().filter(|b| b.testament == "OT").collect();
                            let nt: Vec<_> = bks.iter().filter(|b| b.testament == "NT").collect();
                            rsx! {
                                optgroup { label: "Old Testament",
                                    for b in ot { option { value: "{b.id}", selected: browse_book_id() == b.id, "{b.name}" } }
                                }
                                optgroup { label: "New Testament",
                                    for b in nt { option { value: "{b.id}", selected: browse_book_id() == b.id, "{b.name}" } }
                                }
                            }
                        }
                    }
                    if browse_total() > 0 {
                        div { class: "browse-chapter-strip",
                            for ch in 1..=browse_total() {
                                button {
                                    key: "{ch}",
                                    class: if ch == browse_chapter() { "bch-chip active" } else { "bch-chip" },
                                    onclick: move |_| { browse_chapter.set(ch); staged_verse.set(0); staged_text.set(String::new()); staged_ref.set(String::new()); },
                                    "{ch}"
                                }
                            }
                        }
                    }
                }

                // ── Verse Theater ─────────────────────────────────────────────
                div { class: "browse-verse-list",
                    if let Some(data) = browse_data() {
                        for v in data.verses {
                            {
                                let vnum      = v.verse;
                                let vtext_str = v.text.clone();
                                let vref_str  = format!("{} {}:{}", v.book_name, v.chapter, v.verse);
                                let book_id_c = v.book_id; let chapter_c = v.chapter;
                                let is_live   = reference() == vref_str;
                                let is_staged = staged_verse() == vnum && !is_live;
                                let row_class = if is_live { "bvr live" } else if is_staged { "bvr staged" } else { "bvr" };
                                let stage_t   = vtext_str.clone(); let stage_r = vref_str.clone();
                                rsx! {
                                    div {
                                        key: "{vnum}",
                                        class: row_class,
                                        onclick: move |_| { staged_verse.set(vnum); push_verse(vtext_str.clone(), vref_str.clone(), book_id_c, chapter_c, vnum); },
                                        oncontextmenu: move |e| { e.prevent_default(); stage_only(stage_t.clone(), stage_r.clone(), vnum); },
                                        span { class: "bvr-num", "{vnum}" }
                                        span { class: "bvr-text", "{v.text}" }
                                        if is_live { span { class: "bvr-live-pill", "LIVE" } }
                                        if is_staged { span { class: "bvr-staged-pill", "STAGED" } }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "browse-empty", div { class: "spinner" } span { "Loading…" } }
                    }
                }

                div { class: "kb-guide",
                    span { class: "kb-key", "Click" } span { class: "kb-desc", "go live" }
                    span { class: "kb-sep", "·" }
                    span { class: "kb-key", "Right-click" } span { class: "kb-desc", "stage" }
                    span { class: "kb-sep", "·" }
                    span { class: "kb-key", "Space" } span { class: "kb-desc", "push staged" }
                    span { class: "kb-sep", "·" }
                    span { class: "kb-key", "↑↓" } span { class: "kb-desc", "stage" }
                    span { class: "kb-sep", "·" }
                    span { class: "kb-key", "[ ]" } span { class: "kb-desc", "chapter" }
                    span { class: "kb-sep", "·" }
                    span { class: "kb-key", "C" } span { class: "kb-desc", "clear" }
                }
            } // end present-center

            // ══ RIGHT: Operator Monitor ══════════════════════════════════════
            div { class: "present-right",

                // Push zone — always at top
                div { class: "present-push-zone",
                    button {
                        class: "btn-push-primary",
                        disabled: !can_push,
                        onclick: do_push,
                        if loading() { "Sending…" } else { "▶  Push Live" }
                    }
                    div { class: "action-row-secondary",
                        button { class: "btn-push-next", disabled: !can_push, onclick: do_push_next, "Push + Next ▶" }
                        button { class: "btn-ghost", onclick: do_clear, "✕ Clear" }
                    }
                }

                // NOW LIVE card
                if !reference().is_empty() {
                    div { class: "now-live-card",
                        div { class: "nl-top",
                            div { class: "nl-live-indicator",
                                div { class: "nl-dot" }
                                span { class: "nl-badge", "NOW LIVE" }
                            }
                            div { class: "nl-actions",
                                button { class: "nl-btn", title: "Re-push", disabled: !can_push, onclick: do_push, "↑ Re-push" }
                                button { class: "nl-btn nl-btn-clear", title: "Clear", onclick: do_clear, "✕ Clear" }
                            }
                        }
                        div { class: "nl-ref", "{reference()}" }
                        div { class: "nl-text",
                            { verse_text().chars().take(160).collect::<String>() }
                            if verse_text().len() > 160 { "…" } else { "" }
                        }
                    }
                }

                // STAGED card — keyboard-staged verse, not yet live
                if has_staged {
                    div { class: "staged-card",
                        div { class: "staged-top",
                            div { class: "staged-indicator",
                                div { class: "staged-dot" }
                                span { class: "staged-label", "STAGED" }
                            }
                            span { class: "staged-hint", "Space → Push" }
                        }
                        div { class: "staged-ref", "{staged_ref()}" }
                        div { class: "staged-text",
                            { staged_text().chars().take(120).collect::<String>() }
                            if staged_text().len() > 120 { "…" } else { "" }
                        }
                        button {
                            class: "staged-push-btn",
                            onclick: move |_| {
                                let st = staged_text(); let sr = staged_ref();
                                if st.trim().is_empty() { return; }
                                verse_text.set(st.clone()); reference.set(sr.clone());
                                let t2 = st.clone(); let r2 = sr.clone();
                                spawn(async move {
                                    loading.set(true);
                                    match cmd_push_to_all(&t2, &r2).await {
                                        Ok(p) => {
                                            preview_src.set(format!("data:image/png;base64,{}", p.png_b64));
                                            status.set("Live.".to_string());
                                            let mut h = history.write();
                                            h.retain(|i| i.reference != r2);
                                            h.insert(0, QueueItem { text: t2, reference: r2, book_id: 0, chapter: 0, verse: 0, book_name: String::new() });
                                            if h.len() > 20 { h.truncate(20); }
                                        }
                                        Err(err) => status.set(format!("Error: {err}")),
                                    }
                                    loading.set(false);
                                });
                            },
                            "▶  Push Staged Live"
                        }
                    }
                }

                // Up Next from setlist
                if let Some((next_ref, next_text)) = up_next {
                    div { class: "up-next-card",
                        div { class: "un-top",
                            span { class: "un-label", "UP NEXT" }
                            div { class: "un-actions",
                                button { class: "un-btn", title: "Load", onclick: move |_| load_queue_item(q_idx + 1), "Load" }
                                button { class: "un-btn un-btn-push", title: "Push live now", onclick: do_push_next, "Push ▶" }
                            }
                        }
                        div { class: "un-ref", "{next_ref}" }
                        div { class: "un-text",
                            { next_text.chars().take(80).collect::<String>() }
                            if next_text.len() > 80 { "…" } else { "" }
                        }
                    }
                }

                if !status().is_empty() {
                    div { class: "right-status-chip", "{status}" }
                }

                // Preview
                div { class: "preview-section",
                    div { class: "preview-hdr",
                        span { class: "preview-label", "PREVIEW  {cfg().width}×{cfg().height}" }
                        div { class: "preview-size-row",
                            button { class: if preview_size() == 0 { "psz-btn active" } else { "psz-btn" }, onclick: move |_| preview_size.set(0), "S" }
                            button { class: if preview_size() == 1 { "psz-btn active" } else { "psz-btn" }, onclick: move |_| preview_size.set(1), "M" }
                            button { class: if preview_size() == 2 { "psz-btn active" } else { "psz-btn" }, onclick: move |_| preview_size.set(2), "L" }
                        }
                    }
                    div { class: match preview_size() { 0 => "preview-wrap psz-sm", 2 => "preview-wrap psz-lg", _ => "preview-wrap psz-md" },
                        if !preview_src().is_empty() {
                            img { class: "preview-img", src: "{preview_src}", alt: "preview" }
                        } else {
                            div { class: "preview-placeholder", style: "background:{rgba_to_css(&cfg().background)};", span { "Preview appears here" } }
                        }
                    }
                }

                // Configure Output (collapsible)
                div { class: "config-section",
                    button {
                        class: "config-toggle",
                        onclick: move |_| cfg_open.set(!cfg_open()),
                        span { class: "config-toggle-arrow", if cfg_open() { "▲" } else { "▼" } }
                        "Configure Output"
                        if !cfg_open() { span { class: "config-summary", " — {cfg().template.label()}" } }
                    }

                    if cfg_open() {
                        div { class: "config-body",

                            div { class: "cfg-tab-bar",
                                button { class: if cfg_tab()==0 { "cfg-tab active" } else { "cfg-tab" }, onclick: move|_| cfg_tab.set(0), "Layout" }
                                button { class: if cfg_tab()==1 { "cfg-tab active" } else { "cfg-tab" }, onclick: move|_| cfg_tab.set(1), "Background" }
                                button { class: if cfg_tab()==2 { "cfg-tab active" } else { "cfg-tab" }, onclick: move|_| cfg_tab.set(2), "Text" }
                                button { class: if cfg_tab()==3 { "cfg-tab active" } else { "cfg-tab" }, onclick: move|_| cfg_tab.set(3), "Output" }
                            }

                            if cfg_tab() == 0 {
                                div { class: "cfg-tab-body",
                                    div { class: "ctrl-label", "Layout Template" }
                                    div { class: "tmpl-thumb-grid",
                                        button { class: if cfg().template == Template::FullScreen { "tmpl-card active" } else { "tmpl-card" }, onclick: move |_| cfg.write().template = Template::FullScreen,
                                            div { class: "tmpl-preview tt-fullscreen", div { class: "tt-text-lines", div { class: "tt-line tt-line-lg" } div { class: "tt-line tt-line-md" } div { class: "tt-line tt-line-sm" } } }
                                            span { class: "tmpl-card-label", "Full Screen" }
                                        }
                                        button { class: if cfg().template == Template::LowerThird { "tmpl-card active" } else { "tmpl-card" }, onclick: move |_| cfg.write().template = Template::LowerThird,
                                            div { class: "tmpl-preview tt-lowerthird", div { class: "tt-band", div { class: "tt-line tt-line-lg" } div { class: "tt-line tt-line-sm" } } }
                                            span { class: "tmpl-card-label", "Lower Third" }
                                        }
                                        button { class: if cfg().template == Template::LowerThirdAccent { "tmpl-card active" } else { "tmpl-card" }, onclick: move |_| cfg.write().template = Template::LowerThirdAccent,
                                            div { class: "tmpl-preview tt-ltaccent", div { class: "tt-band", div { class: "tt-line tt-line-lg" } div { class: "tt-line tt-line-sm" } } div { class: "tt-accent-bar" } }
                                            span { class: "tmpl-card-label", "LT + Accent " PremiumBadge { tier: "Standard" } }
                                        }
                                        button { class: if cfg().template == Template::LowerThirdSplit { "tmpl-card active" } else { "tmpl-card" }, onclick: move |_| cfg.write().template = Template::LowerThirdSplit,
                                            div { class: "tmpl-preview tt-ltsplit", div { class: "tt-split-ref" } div { class: "tt-band", div { class: "tt-line tt-line-lg" } } }
                                            span { class: "tmpl-card-label", "LT Split " PremiumBadge { tier: "Standard" } }
                                        }
                                        button { class: if cfg().template == Template::CardCenter { "tmpl-card active" } else { "tmpl-card" }, onclick: move |_| cfg.write().template = Template::CardCenter,
                                            div { class: "tmpl-preview tt-card", div { class: "tt-card-box", div { class: "tt-line tt-line-lg" } div { class: "tt-line tt-line-md" } div { class: "tt-line tt-line-sm" } } }
                                            span { class: "tmpl-card-label", "Card Centre " PremiumBadge { tier: "Standard" } }
                                        }
                                        button { class: if cfg().template == Template::MinimalText { "tmpl-card active" } else { "tmpl-card" }, onclick: move |_| cfg.write().template = Template::MinimalText,
                                            div { class: "tmpl-preview tt-minimal", div { class: "tt-line tt-line-lg tt-line-white" } div { class: "tt-line tt-line-md tt-line-white" } div { class: "tt-line tt-line-sm tt-line-dim" } }
                                            span { class: "tmpl-card-label", "Minimal" }
                                        }
                                    }

                                    {
                                        let t = cfg().template.clone();
                                        let show_band   = matches!(t, Template::LowerThird | Template::LowerThirdAccent | Template::LowerThirdSplit | Template::CardCenter);
                                        let show_accent = matches!(t, Template::LowerThirdAccent | Template::LowerThirdSplit | Template::CardCenter);
                                        if show_band {
                                            rsx! {
                                                div { class: "ctrl-group lt-controls",
                                                    div { class: "ctrl-label-row", span { class: "ctrl-label", "Lower Third" } PremiumBadge { tier: "Standard" } }
                                                    div { class: "lt-preview",
                                                        div { class: "lt-prev-bg" }
                                                        div {
                                                            class: "lt-prev-band",
                                                            style: "height:{(cfg().band_height * 100.0) as i32}%; background:{rgba_to_css(&cfg().band_color)};",
                                                            div { class: "lt-prev-verse-line" }
                                                            div { class: "lt-prev-ref-line" }
                                                            if show_accent {
                                                                div { class: "lt-prev-accent", style: "background:{rgba_to_css(&cfg().accent_color)}; height:{cfg().accent_px as i32}px;" }
                                                            }
                                                        }
                                                    }
                                                    SliderRow { label: "Band Height".to_string(), value: cfg().band_height, min: 0.15, max: 0.60, step: 0.01, on_change: move |v: f32| cfg.write().band_height = v }
                                                    ColorRow { label: "Band Colour".to_string(), color: cfg().band_color.clone(), on_change: move |c: Rgba| cfg.write().band_color = c }
                                                    if show_accent {
                                                        ColorRow { label: "Accent Colour".to_string(), color: cfg().accent_color.clone(), on_change: move |c: Rgba| cfg.write().accent_color = c }
                                                        SliderRow { label: "Accent Height px".to_string(), value: cfg().accent_px, min: 4.0, max: 32.0, step: 1.0, on_change: move |v: f32| cfg.write().accent_px = v }
                                                    }
                                                    div { class: "ctrl-group",
                                                        div { class: "ctrl-label", "Band Gradient" }
                                                        div { class: "canvas-designer-wrap",
                                                            CanvasDesigner {
                                                                design: cfg().band_design.clone().unwrap_or_else(|| crate::commands::BackgroundDesign::solid(cfg().band_color.clone())),
                                                                target: DesignerTarget::Band, band_height: cfg().band_height,
                                                                on_apply: move |d: crate::commands::BackgroundDesign| { cfg.write().band_design = Some(d); },
                                                                on_clear: move |_| { cfg.write().band_design = None; },
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else { rsx! {} }
                                    }

                                    div { class: "ctrl-group",
                                        div { class: "ctrl-label", "Text Position" }
                                        div { class: "pos-row",
                                            button { class: if cfg().position == TextPosition::UpperThird { "pos-btn active" } else { "pos-btn" }, onclick: move |_| cfg.write().position = TextPosition::UpperThird, "Upper" }
                                            button { class: if cfg().position == TextPosition::Center     { "pos-btn active" } else { "pos-btn" }, onclick: move |_| cfg.write().position = TextPosition::Center,     "Center" }
                                            button { class: if cfg().position == TextPosition::LowerThird { "pos-btn active" } else { "pos-btn" }, onclick: move |_| cfg.write().position = TextPosition::LowerThird, "Lower" }
                                        }
                                        SliderRow { label: "Padding".to_string(), value: cfg().padding_x, min: 0.0, max: 0.3, step: 0.01, on_change: move |v: f32| cfg.write().padding_x = v }
                                    }
                                }
                            }

                            if cfg_tab() == 1 {
                                div { class: "cfg-tab-body",
                                    div { class: "bg-type-bar",
                                        button { class: if bg_tab()==0 { "bg-type-btn active" } else { "bg-type-btn" }, onclick: move|_| bg_tab.set(0), "Solid" }
                                        button { class: if bg_tab()==1 { "bg-type-btn active" } else { "bg-type-btn" }, onclick: move|_| bg_tab.set(1), "Gradient" }
                                        button { class: if bg_tab()==2 { "bg-type-btn active" } else { "bg-type-btn" }, onclick: move|_| bg_tab.set(2), "Image" }
                                        button { class: if bg_tab()==3 { "bg-type-btn active" } else { "bg-type-btn" }, onclick: move|_| bg_tab.set(3), "Video" }
                                    }
                                    if bg_tab() == 0 {
                                        div { class: "bg-solid-tab",
                                            div { class: "ctrl-label", "Quick Presets" }
                                            div { class: "bg-preset-row",
                                                button { class: "{sel_black}",  style: "background:#000;",    title: "Black",        onclick: move |_| { cfg.write().background = Rgba{r:0,  g:0,  b:0,  a:255}; cfg.write().bg_design=None; cfg.write().bg_image_b64=None; }, " " }
                                                button { class: "{sel_green}",  style: "background:#00b140;", title: "Chroma Green", onclick: move |_| { cfg.write().background = Rgba{r:0,  g:177,b:64, a:255}; cfg.write().bg_design=None; cfg.write().bg_image_b64=None; }, " " }
                                                button { class: "{sel_blue}",   style: "background:#00f;",    title: "Chroma Blue",  onclick: move |_| { cfg.write().background = Rgba{r:0,  g:0,  b:255,a:255}; cfg.write().bg_design=None; cfg.write().bg_image_b64=None; }, " " }
                                                button { class: "{sel_white}",  style: "background:#fff;",    title: "White",        onclick: move |_| { cfg.write().background = Rgba{r:255,g:255,b:255,a:255}; cfg.write().bg_design=None; cfg.write().bg_image_b64=None; }, " " }
                                                button { class: "{sel_dblue}",  style: "background:#0a0f3c;", title: "Dark Blue",    onclick: move |_| { cfg.write().background = Rgba{r:10, g:15, b:60, a:255}; cfg.write().bg_design=None; cfg.write().bg_image_b64=None; }, " " }
                                                button { class: "{sel_purple}", style: "background:#1e003c;", title: "Deep Purple",  onclick: move |_| { cfg.write().background = Rgba{r:30, g:0,  b:60, a:255}; cfg.write().bg_design=None; cfg.write().bg_image_b64=None; }, " " }
                                                button { class: "preset-btn", style: "background:#1a0a00;", title: "Deep Amber", onclick: move |_| { cfg.write().background = Rgba{r:26, g:10, b:0,  a:255}; cfg.write().bg_design=None; cfg.write().bg_image_b64=None; }, " " }
                                                button { class: "preset-btn", style: "background:#0a1a0a;", title: "Forest",     onclick: move |_| { cfg.write().background = Rgba{r:10, g:26, b:10, a:255}; cfg.write().bg_design=None; cfg.write().bg_image_b64=None; }, " " }
                                            }
                                            ColorRow { label: "Custom Colour".to_string(), color: cfg().background.clone(), on_change: move |c: Rgba| { cfg.write().background = c; cfg.write().bg_design = None; cfg.write().bg_image_b64 = None; } }
                                        }
                                    }
                                    if bg_tab() == 1 {
                                        div { class: "bg-gradient-tab",
                                            div { class: "canvas-designer-wrap",
                                                CanvasDesigner {
                                                    design: cfg().bg_design.clone().unwrap_or_else(|| crate::commands::BackgroundDesign::solid(cfg().background.clone())),
                                                    target: DesignerTarget::Background, band_height: cfg().band_height,
                                                    on_apply: move |d: crate::commands::BackgroundDesign| { cfg.write().bg_design = Some(d); cfg.write().bg_image_b64 = None; },
                                                    on_clear: move |_| { cfg.write().bg_design = None; },
                                                }
                                            }
                                            if cfg().bg_design.is_some() { div { class: "design-active-chip", "✦ Gradient active" } }
                                        }
                                    }
                                    if bg_tab() == 2 {
                                        div { class: "bg-image-tab",
                                            if let Some(img) = cfg().bg_image_b64.clone() {
                                                div { class: "bg-img-preview-wrap",
                                                    img { class: "bg-img-preview", src: "{img}", alt: "background", style: "opacity:{cfg().bg_image_opacity};" }
                                                    button { class: "bg-img-remove", title: "Remove image", onclick: move |_| cfg.write().bg_image_b64 = None, "✕" }
                                                }
                                                div { class: "bg-img-controls",
                                                    SliderRow { label: "Opacity".to_string(), value: cfg().bg_image_opacity, min: 0.1, max: 1.0, step: 0.05, on_change: move |v: f32| cfg.write().bg_image_opacity = v }
                                                    div { class: "ctrl-label", "Fit" }
                                                    div { class: "pos-row",
                                                        for (val, lbl) in [("cover","Cover"),("contain","Contain"),("fill","Fill")] {
                                                            { let v = val.to_string(); let is_active = cfg().bg_image_fit == val; rsx! { button { class: if is_active { "pos-btn active" } else { "pos-btn" }, onclick: move |_| cfg.write().bg_image_fit = v.clone(), "{lbl}" } } }
                                                        }
                                                    }
                                                }
                                            } else {
                                                div { class: "bg-img-empty", div { class: "bg-img-empty-icon", "▣" } p { "No image selected." } p { class: "bg-img-empty-hint", "Supports PNG, JPG, WEBP." } }
                                            }
                                            button {
                                                class: "btn-primary full-w", disabled: bg_image_picking(),
                                                onclick: move |_| { spawn(async move { bg_image_picking.set(true); if let Some(img) = cmd_pick_background_image().await { cfg.write().bg_image_b64 = Some(img); cfg.write().bg_design = None; } bg_image_picking.set(false); }); },
                                                if bg_image_picking() { "Loading…" } else if cfg().bg_image_b64.is_some() { "Replace Image…" } else { "Choose Image…" }
                                            }
                                        }
                                    }
                                    if bg_tab() == 3 {
                                        div { class: "bg-video-tab",
                                            div { class: "bg-video-teaser",
                                                div { class: "bg-video-icon", "▶" }
                                                div { class: "bg-video-info",
                                                    div { class: "bg-video-title", "Video Backgrounds " PremiumBadge { tier: "Standard" } }
                                                    p { class: "bg-video-desc", "Loop MP4 videos behind your Scripture text." }
                                                }
                                            }
                                            PremiumTeaser { icon: "⬡", title: "Cinematic Motion Packs", desc: "Curated worship motion background library.", tier: "Standard" }
                                            PremiumTeaser { icon: "▣", title: "Custom Video Import", desc: "Use your own MP4 or MOV files as looping backgrounds.", tier: "Standard" }
                                        }
                                    }
                                }
                            }

                            if cfg_tab() == 2 {
                                div { class: "cfg-tab-body",
                                    div { class: "typo-preview",
                                        div {
                                            class: "typo-prev-text",
                                            style: "color: {rgba_to_css(&cfg().verse_color)}; font-size: {(cfg().verse_font_size * 0.18) as i32}px; font-family: {match cfg().verse_font_family.as_str() { \"sans\" => \"Inter,system-ui,sans-serif\", \"mono\" => \"Fira Mono,Consolas,monospace\", _ => \"Georgia,'Times New Roman',serif\" }}; font-weight: {if cfg().verse_bold { \"700\" } else { \"400\" }}; font-style: {if cfg().verse_italic { \"italic\" } else { \"normal\" }}; text-align: {cfg().text_align}; line-height: {cfg().line_spacing}; letter-spacing: {cfg().letter_spacing}em; text-shadow: {if cfg().text_shadow { \"0 2px 8px rgba(0,0,0,0.8)\" } else { \"none\" }};",
                                            "For God so loved the world, that he gave his only begotten Son…"
                                        }
                                        div { class: "typo-prev-ref", style: "color:{rgba_to_css(&cfg().reference_color)}; text-align:{cfg().text_align};", "— John 3:16" }
                                    }
                                    div { class: "ctrl-group",
                                        div { class: "ctrl-label", "Font Family" }
                                        div { class: "font-family-row",
                                            for (val, lbl, sample) in [("serif","Serif","Aa"),("sans","Sans","Aa"),("mono","Mono","Aa")] {
                                                { let v = val.to_string(); let is_active = cfg().verse_font_family == val; let fs = match val { "sans" => "font-family:Inter,system-ui,sans-serif;", "mono" => "font-family:'Fira Mono',Consolas,monospace;", _ => "font-family:Georgia,'Times New Roman',serif;" };
                                                  rsx! { button { class: if is_active { "font-choice active" } else { "font-choice" }, style: "{fs}", onclick: move |_| cfg.write().verse_font_family = v.clone(), span { class: "fc-sample", style: "{fs}", "{sample}" } span { class: "fc-label", "{lbl}" } } } }
                                            }
                                        }
                                    }
                                    div { class: "ctrl-group",
                                        div { class: "ctrl-label", "Style" }
                                        div { class: "text-style-row",
                                            button { class: if cfg().verse_bold { "style-btn active" } else { "style-btn" }, style: "font-weight:700;", onclick: move |_| cfg.write().verse_bold = !cfg().verse_bold, "B" }
                                            button { class: if cfg().verse_italic { "style-btn active" } else { "style-btn" }, style: "font-style:italic;", onclick: move |_| cfg.write().verse_italic = !cfg().verse_italic, "I" }
                                            button { class: if cfg().text_shadow { "style-btn active" } else { "style-btn" }, onclick: move |_| cfg.write().text_shadow = !cfg().text_shadow, "Shadow" }
                                        }
                                    }
                                    div { class: "ctrl-group",
                                        div { class: "ctrl-label", "Alignment" }
                                        div { class: "pos-row",
                                            button { class: if cfg().text_align=="left"   { "pos-btn active" } else { "pos-btn" }, onclick: move |_| cfg.write().text_align="left".to_string(),   "Left" }
                                            button { class: if cfg().text_align=="center" { "pos-btn active" } else { "pos-btn" }, onclick: move |_| cfg.write().text_align="center".to_string(), "Center" }
                                            button { class: if cfg().text_align=="right"  { "pos-btn active" } else { "pos-btn" }, onclick: move |_| cfg.write().text_align="right".to_string(),  "Right" }
                                        }
                                    }
                                    div { class: "ctrl-group",
                                        div { class: "ctrl-label", "Verse Text" }
                                        ColorRow { label: "Colour".to_string(), color: cfg().verse_color.clone(), on_change: move |c: Rgba| cfg.write().verse_color = c }
                                        SliderRow { label: "Font Size".to_string(), value: cfg().verse_font_size, min: 24.0, max: 140.0, step: 2.0, on_change: move |v: f32| cfg.write().verse_font_size = v }
                                        SliderRow { label: "Line Spacing".to_string(), value: cfg().line_spacing, min: 1.0, max: 2.2, step: 0.05, on_change: move |v: f32| cfg.write().line_spacing = v }
                                        SliderRow { label: "Letter Spacing em".to_string(), value: cfg().letter_spacing, min: -0.05, max: 0.20, step: 0.01, on_change: move |v: f32| cfg.write().letter_spacing = v }
                                    }
                                    div { class: "ctrl-group",
                                        div { class: "ctrl-label", "Reference Line" }
                                        ColorRow { label: "Colour".to_string(), color: cfg().reference_color.clone(), on_change: move |c: Rgba| cfg.write().reference_color = c }
                                        SliderRow { label: "Font Size".to_string(), value: cfg().reference_font_size, min: 18.0, max: 80.0, step: 2.0, on_change: move |v: f32| cfg.write().reference_font_size = v }
                                        div { class: "ctrl-row", span { class: "ctrl-row-label", "Show Reference" } input { r#type: "checkbox", checked: cfg().show_reference, onchange: move |e| cfg.write().show_reference = e.checked() } }
                                    }
                                }
                            }

                            if cfg_tab() == 3 {
                                div { class: "cfg-tab-body",
                                    div { class: "ctrl-group",
                                        div { class: "ctrl-label", "NDI Source Name" }
                                        input { class: "search-input", value: "{cfg().ndi_name}", oninput: move |e| cfg.write().ndi_name = e.value() }
                                    }
                                    div { class: "ctrl-group",
                                        div { class: "ctrl-label", "Resolution" }
                                        div { class: "res-row",
                                            button { class: if cfg().width==1920 { "pos-btn active" } else { "pos-btn" }, onclick: move |_| { cfg.write().width=1920; cfg.write().height=1080; }, "1920×1080" }
                                            button { class: if cfg().width==1280 { "pos-btn active" } else { "pos-btn" }, onclick: move |_| { cfg.write().width=1280; cfg.write().height=720;  }, "1280×720" }
                                            button { class: if cfg().width==3840 { "pos-btn active" } else { "pos-btn" }, onclick: move |_| { cfg.write().width=3840; cfg.write().height=2160; }, "4K " PremiumBadge { tier: "Standard" } }
                                        }
                                    }
                                }
                            }

                            div { class: "cfg-footer",
                                button {
                                    class: "btn-primary full-w",
                                    onclick: move |_| { let c = cfg(); spawn(async move { match cmd_set_present_config(&c).await { Ok(_) => status.set("Config saved.".to_string()), Err(e) => status.set(format!("Save failed: {e}")), } }); },
                                    "Save Config"
                                }
                            }
                        }
                    }
                }
            } // end present-right
        }
    }
}

// ── sub-components ────────────────────────────────────────────────────────────

#[component]
fn ColorRow(label: String, color: Rgba, on_change: EventHandler<Rgba>) -> Element {
    let hex = rgba_to_hex(&color);
    rsx! {
        div { class: "ctrl-row",
            span { class: "ctrl-row-label", "{label}" }
            div { class: "color-preview", style: "background:{hex};" }
            input { r#type: "color", class: "color-picker", value: "{hex}", onchange: move |e| { if let Some(c) = hex_to_rgba(&e.value()) { on_change.call(c); } } }
            span { class: "color-hex", "{hex}" }
        }
    }
}

#[component]
fn SliderRow(label: String, value: f32, min: f32, max: f32, step: f32, on_change: EventHandler<f32>) -> Element {
    rsx! {
        div { class: "ctrl-row",
            span { class: "ctrl-row-label", "{label}" }
            input { r#type: "range", class: "slider", min: "{min}", max: "{max}", step: "{step}", value: "{value}", oninput: move |e| { if let Ok(v) = e.value().parse::<f32>() { on_change.call(v); } } }
            span { class: "slider-val", "{value:.1}" }
        }
    }
}
