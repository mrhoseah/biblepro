#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::commands::*;

// ── reader view ───────────────────────────────────────────────────────────────

#[component]
pub fn ReaderView(books: Signal<Vec<Book>>, translations: Signal<Vec<Translation>>) -> Element {
    let mut selected_translation = use_signal(|| String::new());
    let mut selected_book        = use_signal(|| 0i32);
    let mut selected_chapter     = use_signal(|| 1i32);
    let mut chapter_data         = use_signal(|| Option::<ChapterInfo>::None);
    let mut sidebar_open         = use_signal(|| true);
    let mut initial_loaded       = use_signal(|| false);
    let mut total_chapters       = use_signal(|| 0i32);

    // Study state
    let mut highlights     = use_signal(|| Vec::<Highlight>::new());
    let mut notes          = use_signal(|| Vec::<Note>::new());
    let mut selected_verse = use_signal(|| Option::<i32>::None);
    let mut note_draft     = use_signal(|| String::new());
    let mut study_sets     = use_signal(|| Vec::<StudySet>::new());
    let mut note_saving    = use_signal(|| false);

    // Right panel — translation comparison
    let mut panel_open:   Signal<bool>                   = use_signal(|| false);
    let mut cmp_ref:      Signal<Option<String>>         = use_signal(|| None);
    let mut cmp_results:  Signal<Vec<(String, String)>>  = use_signal(Vec::new);

    // Flash message for "Present This Verse"
    let mut present_flash = use_signal(|| String::new());

    // Auto-pick first translation and open Genesis 1 — runs once when translations load
    use_effect(move || {
        let list = translations();
        if !initial_loaded() {
            if let Some(first) = list.first() {
                selected_translation.set(first.id.clone());
                selected_book.set(1);
                selected_chapter.set(1);
                initial_loaded.set(true);
            }
        }
    });

    use_effect(move || {
        let t = selected_translation();
        let b = selected_book();
        let c = selected_chapter();
        if t.is_empty() || b == 0 { return; }
        spawn(async move {
            let info = cmd_get_chapter(&t, b, c).await;
            if let Some(ref d) = info { total_chapters.set(d.total_chapters); }
            chapter_data.set(info);
            highlights.set(cmd_get_chapter_highlights(b, c).await);
            notes.set(cmd_get_chapter_notes(b, c).await);
            selected_verse.set(None);
        });
    });

    use_effect(move || {
        spawn(async move { study_sets.set(cmd_get_study_sets().await); });
    });

    // Fetch verse from all translations when comparison panel opens
    use_effect(move || {
        let Some(r) = cmp_ref() else { cmp_results.set(vec![]); return; };
        let ts = translations();
        spawn(async move {
            let mut out = vec![];
            for t in ts {
                if let Some(v) = cmd_search_by_reference(&t.id, &r).await {
                    out.push((t.abbreviation.clone(), v.text.clone()));
                }
            }
            cmp_results.set(out);
        });
    });

    rsx! {
        div { class: "reader-layout",

            // ── LEFT SIDEBAR ──────────────────────────────────────────────────
            aside {
                class: if sidebar_open() { "sidebar" } else { "sidebar collapsed" },
                button {
                    class: "sidebar-toggle",
                    title: if sidebar_open() { "Collapse" } else { "Expand" },
                    onclick: move |_| sidebar_open.set(!sidebar_open()),
                    if sidebar_open() { "◀" } else { "▶" }
                }

                if sidebar_open() {
                    // Translation chips
                    div { class: "sidebar-section",
                        label { class: "sidebar-label", "Translation" }
                        if translations().is_empty() {
                            div { class: "trans-empty-hint", "No data — open Library" }
                        } else {
                            div { class: "trans-switcher",
                                for t in translations() {
                                    {
                                        let tid = t.id.clone();
                                        let is_active = selected_translation() == t.id;
                                        rsx! {
                                            button {
                                                class: if is_active { "trans-pill active" } else { "trans-pill" },
                                                title: "{t.name}  ·  {t.language}",
                                                onclick: move |_| {
                                                    selected_translation.set(tid.clone());
                                                    selected_chapter.set(1);
                                                },
                                                span { class: "pill-abbr", "{t.abbreviation}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Book list
                    div { class: "sidebar-section book-list",
                        div { class: "testament-label", "Old Testament" }
                        for b in books().into_iter().filter(|b| b.testament == "OT") {
                            BookItem {
                                key: "{b.id}",
                                book: b.clone(),
                                selected: selected_book() == b.id,
                                on_select: move |id: i32| {
                                    selected_book.set(id);
                                    selected_chapter.set(1);
                                }
                            }
                        }
                        div { class: "testament-label", "New Testament" }
                        for b in books().into_iter().filter(|b| b.testament == "NT") {
                            BookItem {
                                key: "{b.id}",
                                book: b.clone(),
                                selected: selected_book() == b.id,
                                on_select: move |id: i32| {
                                    selected_book.set(id);
                                    selected_chapter.set(1);
                                }
                            }
                        }
                    }

                    // Chapter strip — direct chapter jump
                    if selected_book() != 0 && total_chapters() > 0 {
                        ChapterStrip {
                            total_chapters: total_chapters(),
                            selected_chapter: selected_chapter(),
                            on_select: move |ch: i32| selected_chapter.set(ch),
                        }
                    }
                }
            }

            // ── CENTER — primary reading area ─────────────────────────────────
            div { class: "reader-main",
                if !present_flash().is_empty() {
                    div { class: "present-flash", "{present_flash}" }
                }

                if let Some(info) = chapter_data() {
                    ChapterNav {
                        book_name: info.book_name.clone(),
                        chapter: info.chapter,
                        total_chapters: info.total_chapters,
                        on_prev: move |_| {
                            let c = selected_chapter();
                            if c > 1 { selected_chapter.set(c - 1); }
                        },
                        on_next: move |_| {
                            let c = selected_chapter();
                            let max = chapter_data().map(|d| d.total_chapters).unwrap_or(1);
                            if c < max { selected_chapter.set(c + 1); }
                        },
                    }
                    div { class: "reader-body",
                        div { class: "verses",
                            for v in info.verses {
                                {
                                    let vid          = v.verse;
                                    let book_id      = v.book_id;
                                    let chapter      = v.chapter;
                                    let verse_text1  = v.text.clone();
                                    let verse_text2  = verse_text1.clone();
                                    let book_name1   = v.book_name.clone();
                                    let book_name2   = book_name1.clone();
                                    let ref_str      = format!("{} {}:{}", v.book_name, v.chapter, v.verse);
                                    let hl_color     = highlights.read().iter()
                                        .find(|h| h.verse == vid).map(|h| h.color.clone());
                                    let has_note     = notes.read().iter().any(|n| n.verse == vid);
                                    let is_selected  = selected_verse() == Some(vid);
                                    let existing_note = notes.read().iter()
                                        .find(|n| n.verse == vid)
                                        .map(|n| n.body.clone())
                                        .unwrap_or_default();
                                    let sets_snap: Vec<(i64, String)> = study_sets.read()
                                        .iter().map(|s| (s.id, s.name.clone())).collect();

                                    rsx! {
                                        StudyVerseRow {
                                            key: "{v.id}",
                                            verse: v,
                                            highlight_color: hl_color,
                                            has_note,
                                            is_selected,
                                            existing_note,
                                            study_sets: sets_snap,
                                            on_select: move |_| {
                                                if selected_verse() == Some(vid) {
                                                    selected_verse.set(None);
                                                } else {
                                                    selected_verse.set(Some(vid));
                                                    let existing = notes.read().iter()
                                                        .find(|n| n.verse == vid)
                                                        .map(|n| n.body.clone())
                                                        .unwrap_or_default();
                                                    note_draft.set(existing);
                                                }
                                            },
                                            on_highlight: move |color: Option<String>| {
                                                spawn(async move {
                                                    match color {
                                                        Some(c) => cmd_set_highlight(book_id, chapter, vid, c).await,
                                                        None    => cmd_remove_highlight(book_id, chapter, vid).await,
                                                    }
                                                    highlights.set(cmd_get_chapter_highlights(book_id, chapter).await);
                                                });
                                            },
                                            on_bookmark: move |_| {
                                                let text  = verse_text1.clone();
                                                let bname = book_name1.clone();
                                                spawn(async move {
                                                    cmd_toggle_bookmark(book_id, chapter, vid, bname, text).await;
                                                });
                                            },
                                            on_save_note: move |body: String| {
                                                note_saving.set(true);
                                                spawn(async move {
                                                    cmd_save_note(book_id, chapter, vid, body).await;
                                                    notes.set(cmd_get_chapter_notes(book_id, chapter).await);
                                                    note_saving.set(false);
                                                });
                                            },
                                            on_add_to_set: move |set_id: i64| {
                                                let text  = verse_text2.clone();
                                                let bname = book_name2.clone();
                                                spawn(async move {
                                                    cmd_add_to_study_set(set_id, book_id, chapter, vid, bname, text).await;
                                                });
                                            },
                                            on_present: move |(text, rref): (String, String)| {
                                                let t2 = text.clone(); let r2 = rref.clone();
                                                spawn(async move {
                                                    match cmd_ndi_push_verse(&t2, &r2).await {
                                                        Ok(_)  => present_flash.set(format!("▶ Sent live — {r2}")),
                                                        Err(e) => present_flash.set(format!("⚠ {e}")),
                                                    }
                                                });
                                            },
                                            on_compare: move |_| {
                                                cmp_ref.set(Some(ref_str.clone()));
                                                panel_open.set(true);
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if selected_book() == 0 && translations().is_empty() {
                    GetStarted {}
                } else {
                    div { class: "reader-placeholder",
                        div { class: "spinner" }
                        p { "Loading…" }
                    }
                }
            }

            // ── RIGHT PANEL — translation comparison ──────────────────────────
            if panel_open() {
                aside { class: "compare-panel",
                    div { class: "cp-header",
                        span { class: "cp-title",
                            if let Some(r) = cmp_ref() { "{r}" } else { "Compare" }
                        }
                        button {
                            class: "cp-close",
                            onclick: move |_| { panel_open.set(false); cmp_ref.set(None); },
                            "✕"
                        }
                    }
                    div { class: "cp-body",
                        if cmp_results().is_empty() {
                            div { class: "cp-loading",
                                div { class: "spinner" }
                                span { "Loading translations…" }
                            }
                        }
                        for (abbr, text) in cmp_results() {
                            div { class: "cp-entry",
                                div { class: "cp-abbr", "{abbr}" }
                                p { class: "cp-text", "{text}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── sub-components ────────────────────────────────────────────────────────────

#[component]
fn GetStarted() -> Element {
    rsx! {
        div { class: "get-started",
            div { class: "gs-logo", "✝" }
            h1 { class: "gs-title", "Welcome to BiblePro" }
            p { class: "gs-sub", "Offline-first Bible reading and live presentation" }
            div { class: "gs-steps",
                div { class: "gs-step",
                    div { class: "gs-step-num", "1" }
                    div { class: "gs-step-body",
                        strong { "Download a Translation" }
                        p { "Go to Library → Download Translations. Choose KJV, NIV, Swahili, or any of 2,500+ translations." }
                    }
                }
                div { class: "gs-step",
                    div { class: "gs-step-num", "2" }
                    div { class: "gs-step-body",
                        strong { "Read & Search" }
                        p { "Select a book in the sidebar or use Search to jump to any verse by reference or keyword." }
                    }
                }
                div { class: "gs-step",
                    div { class: "gs-step-num", "3" }
                    div { class: "gs-step-body",
                        strong { "Present via NDI" }
                        p { "Open Present to push styled verse overlays to any NDI-capable video mixer, projector, or display." }
                    }
                }
            }
        }
    }
}

#[component]
fn BookItem(book: Book, selected: bool, on_select: EventHandler<i32>) -> Element {
    let id = book.id;
    rsx! {
        div {
            class: if selected { "book-item selected" } else { "book-item" },
            onclick: move |_| on_select.call(id),
            "{book.name}"
        }
    }
}

#[component]
fn ChapterStrip(total_chapters: i32, selected_chapter: i32, on_select: EventHandler<i32>) -> Element {
    rsx! {
        div { class: "chapter-strip",
            div { class: "sidebar-label", "Chapter" }
            div { class: "chapter-chips",
                for ch in 1..=total_chapters {
                    button {
                        key: "{ch}",
                        class: if ch == selected_chapter { "ch-chip active" } else { "ch-chip" },
                        onclick: move |_| on_select.call(ch),
                        "{ch}"
                    }
                }
            }
        }
    }
}

#[component]
fn ChapterNav(
    book_name: String,
    chapter: i32,
    total_chapters: i32,
    on_prev: EventHandler<MouseEvent>,
    on_next: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "chapter-nav",
            button {
                class: "nav-arrow",
                disabled: chapter <= 1,
                onclick: move |e| on_prev.call(e),
                "‹"
            }
            div { class: "chapter-title-block",
                h2 { class: "chapter-title", "{book_name}" }
                span { class: "chapter-num", "Chapter {chapter}" }
            }
            button {
                class: "nav-arrow",
                disabled: chapter >= total_chapters,
                onclick: move |e| on_next.call(e),
                "›"
            }
        }
    }
}

const HIGHLIGHT_COLORS: &[(&str, &str)] = &[
    ("yellow", "#f9e94e"),
    ("orange", "#f4a030"),
    ("green",  "#5bcf72"),
    ("blue",   "#5bb4f4"),
    ("pink",   "#f48fb1"),
];

#[component]
fn StudyVerseRow(
    verse:           Verse,
    highlight_color: Option<String>,
    has_note:        bool,
    is_selected:     bool,
    existing_note:   String,
    study_sets:      Vec<(i64, String)>,
    on_select:       EventHandler<()>,
    on_highlight:    EventHandler<Option<String>>,
    on_bookmark:     EventHandler<()>,
    on_save_note:    EventHandler<String>,
    on_add_to_set:   EventHandler<i64>,
    on_present:      EventHandler<(String, String)>,
    on_compare:      EventHandler<()>,
) -> Element {
    let mut note_text = use_signal(|| existing_note.clone());
    let mut show_sets = use_signal(|| false);

    let en = existing_note.clone();
    use_effect(move || { note_text.set(en.clone()); });

    let hl_style = highlight_color.as_deref()
        .and_then(|c| HIGHLIGHT_COLORS.iter().find(|(n, _)| *n == c))
        .map(|(_, hex)| format!("background:{}40;", hex))
        .unwrap_or_default();

    let vtext = verse.text.clone();
    let vref  = format!("{} {}:{}", verse.book_name, verse.chapter, verse.verse);
    let vref2 = vref.clone();

    rsx! {
        div {
            class: if is_selected { "verse-row selected" } else { "verse-row" },
            style: "{hl_style}",
            div { class: "verse-line",
                span {
                    class: "verse-num",
                    title: "Click to study this verse",
                    onclick: move |_| on_select.call(()),
                    "{verse.verse}"
                    if has_note { span { class: "note-dot", title: "Has note" } }
                }
                span { class: "verse-text", "{verse.text}" }
            }
            div { class: "verse-actions",
                button {
                    class: "va-btn va-present",
                    title: "Push this verse live",
                    onclick: move |e| { e.stop_propagation(); on_present.call((vtext.clone(), vref.clone())); },
                    "▶"
                }
                button {
                    class: "va-btn va-compare",
                    title: "Compare translations",
                    onclick: move |e| { e.stop_propagation(); on_compare.call(()); },
                    "⇄"
                }
            }
        }

        if is_selected {
            div { class: "verse-toolbar",
                div { class: "toolbar-section",
                    span { class: "toolbar-label", "Highlight" }
                    div { class: "hl-swatches",
                        for (name, hex) in HIGHLIGHT_COLORS {
                            {
                                let name_owned = name.to_string();
                                let is_active  = highlight_color.as_deref() == Some(name);
                                rsx! {
                                    button {
                                        class: if is_active { "hl-swatch active" } else { "hl-swatch" },
                                        style: "background:{hex};",
                                        title: "{name}",
                                        onclick: move |_| {
                                            if is_active {
                                                on_highlight.call(None);
                                            } else {
                                                on_highlight.call(Some(name_owned.clone()));
                                            }
                                        },
                                    }
                                }
                            }
                        }
                        if highlight_color.is_some() {
                            button {
                                class: "hl-clear",
                                title: "Remove highlight",
                                onclick: move |_| on_highlight.call(None),
                                "✕"
                            }
                        }
                    }
                }

                div { class: "toolbar-actions",
                    button {
                        class: "toolbar-btn toolbar-btn-present",
                        title: "Push this verse to NDI output",
                        onclick: move |_| on_present.call((verse.text.clone(), vref2.clone())),
                        "▶ Present live"
                    }
                    button {
                        class: "toolbar-btn",
                        onclick: move |_| on_bookmark.call(()),
                        "Bookmark"
                    }
                    if !study_sets.is_empty() {
                        div { class: "sets-dropdown",
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| show_sets.set(!show_sets()),
                                "Add to Set"
                            }
                            if show_sets() {
                                div { class: "sets-popup",
                                    for (set_id, set_name) in study_sets.clone() {
                                        button {
                                            class: "set-popup-item",
                                            onclick: move |_| { on_add_to_set.call(set_id); show_sets.set(false); },
                                            "{set_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "note-editor",
                    textarea {
                        class: "note-textarea",
                        rows: "3",
                        placeholder: "Add a study note…",
                        value: "{note_text}",
                        oninput: move |e| note_text.set(e.value()),
                    }
                    button {
                        class: "btn-primary sm-btn",
                        onclick: move |_| on_save_note.call(note_text()),
                        "Save Note"
                    }
                }
            }
        }
    }
}
