#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::commands::*;
use crate::library::LibraryView;
use crate::license::LicenseView;
use crate::plans::PlansView;
use crate::present::PresentView;
use crate::reader::ReaderView;
use crate::search::SearchView;
use crate::study::StudyView;

const VERSION: &str = "0.1.0";

static CSS: Asset = asset!("/assets/styles.css");

#[derive(Clone, PartialEq)]
pub enum View { Reader, Search, Study, Library, Present, About, Plans, License }

pub fn App() -> Element {
    let mut books        = use_signal(|| Vec::<Book>::new());
    let mut translations = use_signal(|| Vec::<Translation>::new());
    let mut stats        = use_signal(|| Option::<DbStats>::None);
    let mut active_view  = use_signal(|| View::Reader);
    let mut palette_open = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            books.set(cmd_get_books().await);
            translations.set(cmd_get_translations().await);
            stats.set(cmd_get_db_stats().await);
        });
    });

    rsx! {
        link { rel: "stylesheet", href: CSS }
        div {
            class: "app",
            onkeydown: move |e: KeyboardEvent| {
                if e.modifiers().ctrl() && e.key() == Key::Character("k".to_string()) {
                    palette_open.set(true);
                }
                if e.key() == Key::Escape {
                    palette_open.set(false);
                }
            },
            Header { active_view }
            div { class: "workspace",
                match active_view() {
                    View::Reader  => rsx! { ReaderView { books, translations } },
                    View::Search  => rsx! { SearchView { books, translations } },
                    View::Study   => rsx! { StudyView {} },
                    View::Library => rsx! { LibraryView { stats, translations } },
                    View::Present => rsx! { PresentView {} },
                    View::About   => rsx! { AboutView {} },
                    View::Plans   => rsx! { PlansView {} },
                    View::License => rsx! { LicenseView {} },
                }
            }
            StatusBar { stats }

            if palette_open() {
                CommandPalette {
                    translations,
                    on_close: move |_| palette_open.set(false),
                    on_navigate: move |v: View| { active_view.set(v); palette_open.set(false); },
                }
            }
        }
    }
}

// ── header ────────────────────────────────────────────────────────────────────

#[component]
fn Header(mut active_view: Signal<View>) -> Element {
    rsx! {
        header { class: "app-header",
            div { class: "header-brand",
                span { class: "brand", "BiblePro" }
                span { class: "brand-version", "v{VERSION}" }
            }
            nav { class: "nav",
                NavBtn { label: "Read",    icon: "▣", active: active_view() == View::Reader,
                    onclick: move |_| active_view.set(View::Reader) }
                NavBtn { label: "Search",  icon: "⌕", active: active_view() == View::Search,
                    onclick: move |_| active_view.set(View::Search) }
                NavBtn { label: "Study",   icon: "✎", active: active_view() == View::Study,
                    onclick: move |_| active_view.set(View::Study) }
                NavBtn { label: "Library", icon: "▤", active: active_view() == View::Library,
                    onclick: move |_| active_view.set(View::Library) }
                NavBtn { label: "Present", icon: "⬡", active: active_view() == View::Present,
                    onclick: move |_| active_view.set(View::Present) }
            }
            div { class: "header-end",
                button {
                    class: if active_view() == View::Plans { "upgrade-btn active" } else { "upgrade-btn" },
                    title: "Plans & Pricing",
                    onclick: move |_| active_view.set(View::Plans),
                    "✦ Upgrade"
                }
                button {
                    class: if active_view() == View::License { "lic-header-btn active" } else { "lic-header-btn" },
                    title: "License & Activation",
                    onclick: move |_| active_view.set(View::License),
                    "⊙ License"
                }
                button {
                    class: if active_view() == View::About { "about-btn active" } else { "about-btn" },
                    title: "About & Support",
                    onclick: move |_| active_view.set(View::About),
                    "?"
                }
            }
        }
    }
}

#[component]
fn NavBtn(label: &'static str, icon: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: if active { "nav-btn active" } else { "nav-btn" },
            onclick: move |e| onclick.call(e),
            span { class: "nav-icon", "{icon}" }
            span { "{label}" }
        }
    }
}

// ── about ─────────────────────────────────────────────────────────────────────

#[component]
fn AboutView() -> Element {
    rsx! {
        div { class: "about-view",
            div { class: "about-inner",

                div { class: "about-hero",
                    div { class: "about-logo", "✝" }
                    h1 { class: "about-title", "BiblePro" }
                    span { class: "about-version-badge", "v{VERSION}" }
                    p { class: "about-tagline",
                        "Offline-first Bible presentation and study for churches, ministries, and media teams."
                    }
                }

                div { class: "about-columns",
                    div { class: "about-card",
                        h2 { class: "about-card-title", "About" }
                        p {
                            "Built out of real ministry experience and the challenges media teams face during "
                            "live scripture projection, worship presentation, and church livestream production."
                        }
                        p {
                            "Designed to simplify Bible projection, improve worship media workflows, and provide "
                            "a faster, more intelligent experience for churches, conferences, ministries, and media teams."
                        }
                        div { class: "about-features",
                            for feat in [
                                ("⬡", "Offline reliability"),
                                ("▶", "Realtime scripture presentation"),
                                ("✎", "AI-powered Bible study"),
                                ("⬡", "Livestream integration"),
                                ("▣", "Modern worship media workflows"),
                                ("◈", "Simplicity for volunteers"),
                                ("✦", "Performance for professional production"),
                            ] {
                                div { class: "about-feature",
                                    span { class: "about-feature-icon", "{feat.0}" }
                                    span { "{feat.1}" }
                                }
                            }
                        }
                        p { class: "about-footer-note",
                            "Whether serving a small church or a large conference setup — Scripture presentation, "
                            "faster, smoother, and more impactful."
                        }
                    }

                    div { class: "about-card",
                        h2 { class: "about-card-title", "Support" }
                        p { "Need help, feature requests, bug reporting, partnership inquiries, or ministry support?" }
                        div { class: "support-item",
                            span { class: "support-icon", "✉" }
                            div {
                                div { class: "support-label", "Email" }
                                a {
                                    class: "support-link",
                                    href: "mailto:hoseahkplgt@gmail.com",
                                    "hoseahkplgt@gmail.com"
                                }
                            }
                        }
                        div { class: "support-coming",
                            p { class: "support-coming-title", "Coming soon" }
                            for item in ["GitHub Issues","Discord Community","Documentation Portal","Video Tutorials","Feature Request Board"] {
                                div { class: "support-coming-item",
                                    span { class: "coming-dot" }
                                    span { "{item}" }
                                }
                            }
                        }
                    }
                }

                div { class: "about-powered",
                    "Built for churches, ministries, conferences, and worship media teams."
                    br {}
                    span { class: "about-org", "Powered by FGCG — East Gate Chapel." }
                }
            }
        }
    }
}

// ── command palette ───────────────────────────────────────────────────────────

#[derive(Clone)]
struct PaletteAction { label: String, hint: String, kind: PaletteKind }

#[derive(Clone)]
enum PaletteKind { Navigate(View), SearchRef(String) }

#[component]
fn CommandPalette(
    translations: Signal<Vec<Translation>>,
    on_close:     EventHandler<()>,
    on_navigate:  EventHandler<View>,
) -> Element {
    let mut query = use_signal(String::new);
    let mut sel   = use_signal(|| 0usize);

    let q = query().to_lowercase();
    let mut actions: Vec<PaletteAction> = vec![];

    for (label, hint, view) in [
        ("Read",    "Open Bible reader",             View::Reader),
        ("Search",  "Full-text Bible search",        View::Search),
        ("Study",   "Study tools",                   View::Study),
        ("Library", "Import & manage translations",  View::Library),
        ("Present", "NDI presentation control",      View::Present),
        ("License", "License & activation",          View::License),
    ] {
        if q.is_empty() || label.to_lowercase().contains(&q) || hint.to_lowercase().contains(&q) {
            actions.push(PaletteAction { label: label.to_string(), hint: hint.to_string(), kind: PaletteKind::Navigate(view) });
        }
    }

    let looks_like_ref = !q.is_empty()
        && q.chars().any(|c| c.is_alphabetic())
        && q.chars().any(|c| c.is_numeric() || c == ':');
    if looks_like_ref {
        actions.insert(0, PaletteAction {
            label: format!("Jump to {}", query()),
            hint:  "Search this reference in all loaded translations".to_string(),
            kind:  PaletteKind::SearchRef(query()),
        });
    }

    let action_views: Vec<(View, bool)> = actions.iter().map(|a| match &a.kind {
        PaletteKind::Navigate(v)  => (v.clone(), false),
        PaletteKind::SearchRef(_) => (View::Search, true),
    }).collect();
    let action_count = actions.len();
    let sel_view: Option<(View, bool)> = action_views.get(sel()).cloned();

    rsx! {
        div {
            class: "palette-backdrop",
            onclick: move |_| on_close.call(()),
            div {
                class: "palette-modal",
                onclick: move |e| e.stop_propagation(),
                div { class: "palette-input-wrap",
                    span { class: "palette-icon", "Ctrl+K" }
                    input {
                        class: "palette-input",
                        autofocus: true,
                        placeholder: "Navigate or enter a verse reference (e.g. John 3:16)…",
                        value: "{query}",
                        oninput: move |e| { query.set(e.value()); sel.set(0); },
                        onkeydown: {
                            let sv = sel_view.clone();
                            move |e: KeyboardEvent| {
                                match e.key() {
                                    Key::ArrowDown => sel.set((sel() + 1).min(action_count.saturating_sub(1))),
                                    Key::ArrowUp   => sel.set(sel().saturating_sub(1)),
                                    Key::Enter     => { if let Some((v, _)) = sv.clone() { on_navigate.call(v); } }
                                    Key::Escape    => on_close.call(()),
                                    _ => {}
                                }
                            }
                        },
                    }
                }
                div { class: "palette-results",
                    if actions.is_empty() {
                        div { class: "palette-empty", "No matching commands" }
                    }
                    for (i, (action, (dest, is_ref))) in actions.iter().zip(action_views.iter()).enumerate() {
                        {
                            let label     = action.label.clone();
                            let hint      = action.hint.clone();
                            let dest_view = dest.clone();
                            let is_ref    = *is_ref;
                            rsx! {
                                div {
                                    key: "{i}",
                                    class: if sel() == i { "palette-item selected" } else { "palette-item" },
                                    onclick: move |_| on_navigate.call(dest_view.clone()),
                                    div { class: "palette-item-left",
                                        span { class: if is_ref { "palette-item-icon ref-icon" } else { "palette-item-icon" },
                                            if is_ref { "◈" } else { "→" }
                                        }
                                        span { class: "palette-item-label", "{label}" }
                                    }
                                    span { class: "palette-item-hint", "{hint}" }
                                }
                            }
                        }
                    }
                }
                div { class: "palette-footer",
                    span { "↑↓ navigate" }
                    span { "⏎ select" }
                    span { "Esc close" }
                }
            }
        }
    }
}

// ── status bar ────────────────────────────────────────────────────────────────

fn fmt_num(n: i64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::new();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 { out.push(','); }
        out.push(b as char);
    }
    out
}

#[component]
fn StatusBar(stats: Signal<Option<DbStats>>) -> Element {
    rsx! {
        footer { class: "status-bar",
            span { class: "status-left",
                if let Some(s) = stats() {
                    if s.translation_count == 0 {
                        span { "No translations — go to Library to download" }
                    } else {
                        span { "{s.translation_count} translation(s)  ·  {fmt_num(s.verse_count)} verses" }
                    }
                } else {
                    span { "BiblePro" }
                }
            }
            span { class: "status-right", "v{VERSION}" }
        }
    }
}
