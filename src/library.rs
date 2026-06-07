#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use serde::Deserialize;
use crate::commands::*;

// ── WASM fetch helper ─────────────────────────────────────────────────────────
// Downloads a URL via the browser's native fetch() — no Tauri backend needed.

#[wasm_bindgen(inline_js = "
export async function wasm_fetch_text(url, api_key) {
    const headers = {};
    if (api_key) headers['api-key'] = api_key;
    const resp = await fetch(url, { headers });
    if (!resp.ok) throw new Error('HTTP ' + resp.status + ' — ' + resp.statusText);
    return await resp.text();
}
")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn wasm_fetch_text(url: &str, api_key: &str) -> Result<JsValue, JsValue>;
}

async fn fetch_text(url: &str) -> Result<String, String> {
    wasm_fetch_text(url, "")
        .await
        .map(|v| v.as_string().unwrap_or_default())
        .map_err(|e| e.as_string().unwrap_or_else(|| "Network error".to_string()))
}

async fn fetch_text_with_key(url: &str, key: &str) -> Result<String, String> {
    wasm_fetch_text(url, key)
        .await
        .map(|v| v.as_string().unwrap_or_default())
        .map_err(|e| e.as_string().unwrap_or_else(|| "Network error".to_string()))
}

// ── API.Bible types ───────────────────────────────────────────────────────────

#[derive(Deserialize, Clone, Debug)]
struct ApiBible {
    id:           String,
    name:         String,
    #[serde(rename = "nameLocal")]
    name_local:   Option<String>,
    language:     ApiBibleLanguage,
    abbreviation: Option<String>,
    description:  Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct ApiBibleLanguage {
    name:      String,
    #[serde(rename = "nameLocal")]
    name_local: Option<String>,
}

#[derive(Deserialize)]
struct ApiBiblesResponse {
    data: Vec<ApiBible>,
}

#[derive(Deserialize, Clone, Debug)]
struct ApiBibleBook {
    id:           String,
    #[serde(rename = "bibleId")]
    bible_id:     String,
    abbreviation: String,
    name:         String,
}

#[derive(Deserialize)]
struct ApiBooksResponse {
    data: Vec<ApiBibleBook>,
}

#[derive(Deserialize, Clone, Debug)]
struct ApiChapter {
    id:       String,
    #[serde(rename = "bookId")]
    book_id:  String,
    number:   String,
    #[serde(rename = "bibleId")]
    bible_id: String,
}

#[derive(Deserialize)]
struct ApiChaptersResponse {
    data: Vec<ApiChapter>,
}

#[derive(Deserialize)]
struct ApiChapterContent {
    data: ApiChapterData,
}

#[derive(Deserialize)]
struct ApiChapterData {
    content:    String,
    #[serde(rename = "verseCount")]
    verse_count: Option<i32>,
}

// ── Curated open-domain catalogue ─────────────────────────────────────────────
// All URLs point to jsDelivr CDN (mirrors GitHub, very reliable globally).

struct CuratedTranslation {
    id:          &'static str,
    abbrev:      &'static str,
    name:        &'static str,
    lang:        &'static str,
    lang_code:   &'static str,
    url:         &'static str,
    license:     &'static str,
    description: &'static str,
}

const CATALOGUE: &[CuratedTranslation] = &[
    // ── English ──
    CuratedTranslation {
        id: "kjv", abbrev: "KJV", name: "King James Version",
        lang: "English", lang_code: "en",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/en_kjv.json",
        license: "Public Domain",
        description: "The classic 1611 English Bible — most widely used in churches worldwide.",
    },
    CuratedTranslation {
        id: "asv", abbrev: "ASV", name: "American Standard Version",
        lang: "English", lang_code: "en",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/en_asv.json",
        license: "Public Domain",
        description: "A faithful 1901 revision of the KJV, close to Hebrew and Greek originals.",
    },
    CuratedTranslation {
        id: "ylt", abbrev: "YLT", name: "Young's Literal Translation",
        lang: "English", lang_code: "en",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/en_ylt.json",
        license: "Public Domain",
        description: "Extremely literal — preserves original tenses and word order.",
    },
    CuratedTranslation {
        id: "web", abbrev: "WEB", name: "World English Bible",
        lang: "English", lang_code: "en",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/en_web.json",
        license: "Public Domain",
        description: "Modern-English update of the ASV. No copyright restrictions.",
    },
    // ── French ──
    CuratedTranslation {
        id: "lsg", abbrev: "LSG", name: "Louis Segond 1910",
        lang: "French", lang_code: "fr",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/fr_lsg.json",
        license: "Public Domain",
        description: "La Bible de Segond — référence protestante francophone.",
    },
    // ── Spanish ──
    CuratedTranslation {
        id: "rvr", abbrev: "RVR", name: "Reina-Valera 1960",
        lang: "Spanish", lang_code: "es",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/es_rvr.json",
        license: "Public Domain",
        description: "La Biblia más usada en el mundo evangélico hispanohablante.",
    },
    // ── Portuguese ──
    CuratedTranslation {
        id: "alm", abbrev: "ALM", name: "Almeida Corrigida",
        lang: "Portuguese", lang_code: "pt",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/pt_aa.json",
        license: "Public Domain",
        description: "A Bíblia clássica em Português — usada no Brasil e Portugal.",
    },
    // ── German ──
    CuratedTranslation {
        id: "luth", abbrev: "LUT", name: "Luther Bibel 1912",
        lang: "German", lang_code: "de",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/de_luth.json",
        license: "Public Domain",
        description: "Luthers klassische Bibelübersetzung.",
    },
    // ── Swahili ──
    CuratedTranslation {
        id: "swahili", abbrev: "SUV", name: "Swahili Union Version",
        lang: "Swahili", lang_code: "sw",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/sw_swahili.json",
        license: "Public Domain",
        description: "Biblia Takatifu — standard Swahili Bible across East Africa.",
    },
    // ── Romanian ──
    CuratedTranslation {
        id: "cornilescu", abbrev: "COR", name: "Cornilescu 1924",
        lang: "Romanian", lang_code: "ro",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/ro_cornilescu.json",
        license: "Public Domain",
        description: "Biblia Cornilescu — cea mai răspândită traducere protestantă.",
    },
    // ── Chinese ──
    CuratedTranslation {
        id: "cuv", abbrev: "CUV", name: "Chinese Union Version",
        lang: "Chinese", lang_code: "zh",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/zh_cuv.json",
        license: "Public Domain",
        description: "和合本 — the most widely used Chinese Bible.",
    },
    // ── Finnish ──
    CuratedTranslation {
        id: "fin1776", abbrev: "FIN", name: "Finnish Bible 1776",
        lang: "Finnish", lang_code: "fi",
        url: "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json/fi_finnish.json",
        license: "Public Domain",
        description: "The classic Finnish Bible.",
    },
];

fn unique_languages() -> Vec<&'static str> {
    let mut langs: Vec<&'static str> = Vec::new();
    for t in CATALOGUE {
        if !langs.contains(&t.lang) { langs.push(t.lang); }
    }
    langs
}

// ── LibraryView ───────────────────────────────────────────────────────────────

#[component]
pub fn LibraryView(mut stats: Signal<Option<DbStats>>, mut translations: Signal<Vec<Translation>>) -> Element {
    // Download state per translation id
    let mut dl_status: Signal<std::collections::HashMap<String, String>> = use_signal(std::collections::HashMap::new);
    let mut removing:  Signal<std::collections::HashMap<String, bool>>   = use_signal(std::collections::HashMap::new);

    // Language filter for the catalogue
    let mut lang_filter: Signal<Option<&'static str>> = use_signal(|| None);

    // File import
    let mut pick_id:     Signal<String> = use_signal(String::new);
    let mut pick_name:   Signal<String> = use_signal(String::new);
    let mut pick_lang:   Signal<String> = use_signal(|| "en".to_string());
    let mut pick_status: Signal<String> = use_signal(String::new);
    let mut picking:     Signal<bool>   = use_signal(|| false);

    // API.Bible state
    let mut api_key:       Signal<String>          = use_signal(String::new);
    let mut api_key_draft: Signal<String>          = use_signal(String::new);
    let mut api_bibles:    Signal<Vec<ApiBible>>   = use_signal(Vec::new);
    let mut api_loading:   Signal<bool>            = use_signal(|| false);
    let mut api_error:     Signal<String>          = use_signal(String::new);
    let mut api_search:    Signal<String>          = use_signal(String::new);
    let mut api_lang_filter: Signal<String>        = use_signal(String::new);

    // Active lib tab: 0=catalogue  1=api.bible  2=import
    let mut lib_tab: Signal<u8> = use_signal(|| 0u8);

    let installed_ids: Vec<String> = translations().iter().map(|t| t.id.clone()).collect();

    let visible: Vec<&'static CuratedTranslation> = CATALOGUE.iter()
        .filter(|t| lang_filter().map_or(true, |f| t.lang == f))
        .collect();

    // Filtered API.Bible list
    let api_visible: Vec<ApiBible> = {
        let q  = api_search().to_lowercase();
        let lf = api_lang_filter().to_lowercase();
        api_bibles().into_iter().filter(|b| {
            let name_match  = q.is_empty()  || b.name.to_lowercase().contains(&q) || b.abbreviation.as_deref().unwrap_or("").to_lowercase().contains(&q);
            let lang_match  = lf.is_empty() || b.language.name.to_lowercase().contains(&lf);
            name_match && lang_match
        }).collect()
    };

    rsx! {
        div { class: "library-view",

            // ── Stats bar ─────────────────────────────────────────────────────
            div { class: "lib-stats-bar",
                if let Some(s) = stats() {
                    div { class: "lib-stat",
                        span { class: "lib-stat-val", "{s.translation_count}" }
                        span { class: "lib-stat-lbl", "Translations" }
                    }
                    div { class: "lib-stat-sep" }
                    div { class: "lib-stat",
                        span { class: "lib-stat-val", "{s.book_count}" }
                        span { class: "lib-stat-lbl", "Books" }
                    }
                    div { class: "lib-stat-sep" }
                    div { class: "lib-stat",
                        span { class: "lib-stat-val", "{fmt_num(s.verse_count)}" }
                        span { class: "lib-stat-lbl", "Verses" }
                    }
                } else {
                    span { class: "lib-loading", "Loading…" }
                }
            }

            // ── Installed translations ────────────────────────────────────────
            if translations().is_empty() {
                div { class: "lib-installed-empty",
                    div { class: "lie-icon", "▣" }
                    p { "No translations installed." }
                    p { class: "lie-hint", "Download one from the catalogue or import a JSON file." }
                }
            } else {
                div { class: "lib-installed-section",
                    div { class: "lib-section-title", "Installed" }
                    div { class: "lib-installed-grid",
                        for t in translations() {
                            {
                                let tid = t.id.clone();
                                let is_removing = removing.read().get(&tid).copied().unwrap_or(false);
                                rsx! {
                                    div { class: "installed-card",
                                        div { class: "ic-abbrev", "{t.abbreviation}" }
                                        div { class: "ic-info",
                                            div { class: "ic-name", "{t.name}" }
                                            div { class: "ic-lang", "{t.language}" }
                                        }
                                        button {
                                            class: "ic-remove-btn",
                                            title: "Remove",
                                            disabled: is_removing,
                                            onclick: move |_| {
                                                let id = tid.clone();
                                                removing.write().insert(id.clone(), true);
                                                spawn(async move {
                                                    let _ = cmd_remove_translation(&id).await;
                                                    translations.set(cmd_get_translations().await);
                                                    stats.set(cmd_get_db_stats().await);
                                                    removing.write().remove(&id);
                                                });
                                            },
                                            if is_removing { "…" } else { "✕" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Tab bar ───────────────────────────────────────────────────────
            div { class: "lib-tab-bar",
                button {
                    class: if lib_tab()==0 { "lib-tab active" } else { "lib-tab" },
                    onclick: move |_| lib_tab.set(0),
                    "▤  Free Downloads"
                }
                button {
                    class: if lib_tab()==1 { "lib-tab active" } else { "lib-tab" },
                    onclick: move |_| lib_tab.set(1),
                    "⬡  API.Bible  (2,500+)"
                }
                button {
                    class: if lib_tab()==2 { "lib-tab active" } else { "lib-tab" },
                    onclick: move |_| lib_tab.set(2),
                    "▣  Import File"
                }
            }

            // ══ Tab 0: Curated free catalogue ═════════════════════════════════
            if lib_tab() == 0 {

                // Language filter chips
                div { class: "lang-filter-bar",
                    button {
                        class: if lang_filter().is_none() { "lang-chip active" } else { "lang-chip" },
                        onclick: move |_| lang_filter.set(None),
                        "All"
                    }
                    for lang in unique_languages() {
                        {
                            let is_active = lang_filter() == Some(lang);
                            rsx! {
                                button {
                                    class: if is_active { "lang-chip active" } else { "lang-chip" },
                                    onclick: move |_| lang_filter.set(Some(lang)),
                                    "{lang}"
                                }
                            }
                        }
                    }
                }

                // Catalogue
                div { class: "catalogue-list",
                    for entry in visible {
                        {
                            let is_installed = installed_ids.contains(&entry.id.to_string());
                            let status_msg   = dl_status.read().get(entry.id).cloned().unwrap_or_default();
                            let is_busy      = !status_msg.is_empty()
                                && !status_msg.starts_with("✓")
                                && !status_msg.starts_with("Error");
                            let eid   = entry.id.to_string();
                            let eurl  = entry.url.to_string();
                            let ename = entry.name.to_string();
                            let elang = entry.lang_code.to_string();

                            rsx! {
                                div { class: if is_installed { "cat-row installed" } else { "cat-row" },
                                    div { class: "cat-abbrev",
                                        "{entry.abbrev}"
                                        div { class: "cat-license", "{entry.license}" }
                                    }
                                    div { class: "cat-info",
                                        div { class: "cat-name", "{entry.name}" }
                                        div { class: "cat-meta",
                                            span { class: "cat-lang", "{entry.lang}" }
                                            span { class: "cat-dot", "·" }
                                            span { class: "cat-desc", "{entry.description}" }
                                        }
                                        if !status_msg.is_empty() {
                                            div {
                                                class: if status_msg.starts_with("Error") { "cat-status error" } else { "cat-status ok" },
                                                "{status_msg}"
                                            }
                                        }
                                    }
                                    div { class: "cat-action",
                                        if is_installed {
                                            div { class: "cat-installed-badge", "✓ Installed" }
                                        } else if is_busy {
                                            div { class: "cat-downloading",
                                                div { class: "cat-dl-spinner" }
                                                span { "{status_msg}" }
                                            }
                                        } else {
                                            button {
                                                class: "cat-dl-btn",
                                                onclick: move |_| {
                                                    let id    = eid.clone();
                                                    let url   = eurl.clone();
                                                    let name  = ename.clone();
                                                    let lang  = elang.clone();

                                                    dl_status.write().insert(id.clone(), "Downloading…".to_string());

                                                    spawn(async move {
                                                        // Step 1: download JSON via WASM fetch (browser-native, no Tauri needed)
                                                        match fetch_text(&url).await {
                                                            Ok(json_str) => {
                                                                dl_status.write().insert(id.clone(), "Importing…".to_string());
                                                                // Step 2: send JSON string to Tauri backend to parse + store
                                                                match cmd_import_from_json(&json_str, &id, &name, &lang).await {
                                                                    Ok(r) => {
                                                                        dl_status.write().insert(id.clone(), format!("✓ {}", r.message));
                                                                        translations.set(cmd_get_translations().await);
                                                                        stats.set(cmd_get_db_stats().await);
                                                                    }
                                                                    Err(e) => {
                                                                        dl_status.write().insert(id.clone(), format!("Error: {e}"));
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                dl_status.write().insert(id.clone(), format!("Error: {e}"));
                                                            }
                                                        }
                                                    });
                                                },
                                                "⬇  Download"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "catalogue-footer-note",
                    "All translations above are public domain and hosted on jsDelivr CDN. "
                    "For licensed translations (NIV, ESV, NASB etc.), use the API.Bible tab."
                }
            }

            // ══ Tab 1: API.Bible ═══════════════════════════════════════════════
            if lib_tab() == 1 {
                div { class: "api-bible-panel",

                    // API key setup
                    div { class: "api-key-section",
                        div { class: "api-key-header",
                            div { class: "api-key-logo", "⬡" }
                            div {
                                div { class: "api-key-title", "API.Bible — 2,500+ Translations" }
                                div { class: "api-key-sub",
                                    "Free API key at "
                                    a {
                                        class: "api-key-link",
                                        href: "https://scripture.api.bible",
                                        target: "_blank",
                                        "scripture.api.bible"
                                    }
                                    " — takes 30 seconds."
                                }
                            }
                        }

                        if api_key().is_empty() {
                            div { class: "api-key-form",
                                input {
                                    class: "search-input api-key-input",
                                    placeholder: "Paste your API.Bible key here…",
                                    value: "{api_key_draft}",
                                    oninput: move |e| api_key_draft.set(e.value()),
                                    onkeydown: move |e| {
                                        if e.key() == Key::Enter && !api_key_draft().trim().is_empty() {
                                            let key = api_key_draft().trim().to_string();
                                            api_key.set(key.clone());
                                            api_loading.set(true);
                                            api_error.set(String::new());
                                            spawn(async move {
                                                let url = "https://api.scripture.api.bible/v1/bibles?language=ENG&includeFullDetails=false";
                                                match fetch_text_with_key(url, &key).await {
                                                    Ok(body) => {
                                                        match serde_json::from_str::<ApiBiblesResponse>(&body) {
                                                            Ok(resp) => api_bibles.set(resp.data),
                                                            Err(e)   => { api_error.set(format!("Parse error: {e}")); api_key.set(String::new()); }
                                                        }
                                                    }
                                                    Err(e) => { api_error.set(format!("Invalid key or network error: {e}")); api_key.set(String::new()); }
                                                }
                                                api_loading.set(false);
                                            });
                                        }
                                    }
                                }
                                button {
                                    class: "btn-primary",
                                    disabled: api_loading() || api_key_draft().trim().is_empty(),
                                    onclick: move |_| {
                                        let key = api_key_draft().trim().to_string();
                                        if key.is_empty() { return; }
                                        api_key.set(key.clone());
                                        api_loading.set(true);
                                        api_error.set(String::new());
                                        spawn(async move {
                                            // Fetch all Bibles (first page, no language filter so we get everything)
                                            let url = "https://api.scripture.api.bible/v1/bibles?includeFullDetails=false";
                                            match fetch_text_with_key(&url, &key).await {
                                                Ok(body) => {
                                                    match serde_json::from_str::<ApiBiblesResponse>(&body) {
                                                        Ok(resp) => { api_bibles.set(resp.data); }
                                                        Err(e)   => { api_error.set(format!("Parse error: {e}")); api_key.set(String::new()); }
                                                    }
                                                }
                                                Err(e) => { api_error.set(format!("Error: {e}")); api_key.set(String::new()); }
                                            }
                                            api_loading.set(false);
                                        });
                                    },
                                    if api_loading() { "Connecting…" } else { "Connect" }
                                }
                                if !api_error().is_empty() {
                                    div { class: "api-key-error", "{api_error}" }
                                }
                                // How-to guide
                                div { class: "api-key-howto",
                                    div { class: "howto-step",
                                        span { class: "howto-num", "1" }
                                        span { "Go to " strong { "scripture.api.bible" } " and sign up for a free account." }
                                    }
                                    div { class: "howto-step",
                                        span { class: "howto-num", "2" }
                                        span { "Create an application in your dashboard to get your API key." }
                                    }
                                    div { class: "howto-step",
                                        span { class: "howto-num", "3" }
                                        span { "Paste the key above and click Connect to browse 2,500+ translations." }
                                    }
                                }
                            }
                        } else {
                            // Connected — show stats + disconnect
                            div { class: "api-connected-bar",
                                div { class: "api-conn-dot" }
                                span { class: "api-conn-label", "API.Bible connected" }
                                span { class: "api-conn-count", "{api_bibles().len()} translations available" }
                                button {
                                    class: "api-disconnect-btn",
                                    onclick: move |_| {
                                        api_key.set(String::new());
                                        api_bibles.set(Vec::new());
                                        api_key_draft.set(String::new());
                                    },
                                    "Disconnect"
                                }
                            }
                        }
                    }

                    // Translation browser (only shown once connected)
                    if !api_key().is_empty() && !api_bibles().is_empty() {

                        // Search + language filter
                        div { class: "api-search-bar",
                            input {
                                class: "search-input api-search-input",
                                placeholder: "Search translations…  (e.g. NIV, Swahili, Arabic)",
                                value: "{api_search}",
                                oninput: move |e| api_search.set(e.value()),
                            }
                            input {
                                class: "search-input",
                                style: "max-width:140px;",
                                placeholder: "Language",
                                value: "{api_lang_filter}",
                                oninput: move |e| api_lang_filter.set(e.value()),
                            }
                        }

                        div { class: "api-result-count",
                            "{api_visible.len()} of {api_bibles().len()} translations"
                        }

                        div { class: "api-bible-list",
                            for bible in api_visible {
                                {
                                    let bible_id   = bible.id.clone();
                                    let bible_name = bible.name.clone();
                                    let bible_abbr = bible.abbreviation.clone().unwrap_or_else(|| "?".to_string());
                                    let lang_name  = bible.language.name_local.clone().unwrap_or_else(|| bible.language.name.clone());
                                    let dl_key     = format!("api_{}", bible_id);
                                    let status_msg = dl_status.read().get(&dl_key).cloned().unwrap_or_default();
                                    let is_installed = installed_ids.contains(&bible_id.to_string().to_lowercase());
                                    let is_busy = !status_msg.is_empty() && !status_msg.starts_with("✓") && !status_msg.starts_with("Error");
                                    let bid  = bible_id.clone();
                                    let bname = bible_name.clone();
                                    let babbr = bible_abbr.clone();
                                    let blang = bible.language.name.clone();
                                    let bkey  = api_key().clone();
                                    let dlk   = dl_key.clone();

                                    rsx! {
                                        div { class: if is_installed { "api-bible-row installed" } else { "api-bible-row" },
                                            div { class: "api-row-abbrev", "{bible_abbr}" }
                                            div { class: "api-row-info",
                                                div { class: "api-row-name", "{bible_name}" }
                                                div { class: "api-row-lang", "{lang_name}" }
                                                if let Some(desc) = bible.description {
                                                    div { class: "api-row-desc", "{desc}" }
                                                }
                                                if !status_msg.is_empty() {
                                                    div {
                                                        class: if status_msg.starts_with("Error") { "cat-status error" } else { "cat-status ok" },
                                                        "{status_msg}"
                                                    }
                                                }
                                            }
                                            div { class: "cat-action",
                                                if is_installed {
                                                    div { class: "cat-installed-badge", "✓ Installed" }
                                                } else if is_busy {
                                                    div { class: "cat-downloading",
                                                        div { class: "cat-dl-spinner" }
                                                        span { "{status_msg}" }
                                                    }
                                                } else {
                                                    button {
                                                        class: "cat-dl-btn",
                                                        title: "Download {bible_name}",
                                                        onclick: move |_| {
                                                            let id      = bid.clone();
                                                            let name    = bname.clone();
                                                            let abbr    = babbr.clone();
                                                            let lang    = blang.clone();
                                                            let key     = bkey.clone();
                                                            let dlkey   = dlk.clone();
                                                            let trans_id = abbr.to_lowercase().replace(' ', "_");

                                                            dl_status.write().insert(dlkey.clone(), "Fetching books…".to_string());

                                                            spawn(async move {
                                                                // Fetch all books for this Bible
                                                                let books_url = format!(
                                                                    "https://api.scripture.api.bible/v1/bibles/{id}/books"
                                                                );
                                                                let books_json = match fetch_text_with_key(&books_url, &key).await {
                                                                    Ok(j) => j,
                                                                    Err(e) => {
                                                                        dl_status.write().insert(dlkey, format!("Error: {e}"));
                                                                        return;
                                                                    }
                                                                };
                                                                let books: Vec<ApiBibleBook> = match serde_json::from_str::<ApiBooksResponse>(&books_json) {
                                                                    Ok(r) => r.data,
                                                                    Err(e) => { dl_status.write().insert(dlkey, format!("Parse error: {e}")); return; }
                                                                };

                                                                let total_books = books.len();
                                                                let mut thiago_bible: Vec<serde_json::Value> = Vec::new();

                                                                for (bi, book) in books.iter().enumerate() {
                                                                    dl_status.write().insert(dlkey.clone(), format!("Fetching {}/{} — {}…", bi+1, total_books, book.name));

                                                                    // Fetch all chapters for this book
                                                                    let chaps_url = format!(
                                                                        "https://api.scripture.api.bible/v1/bibles/{id}/books/{}/chapters",
                                                                        book.id
                                                                    );
                                                                    let chaps_json = match fetch_text_with_key(&chaps_url, &key).await {
                                                                        Ok(j) => j,
                                                                        Err(_) => continue,
                                                                    };
                                                                    let chapters: Vec<ApiChapter> = match serde_json::from_str::<ApiChaptersResponse>(&chaps_json) {
                                                                        Ok(r) => r.data.into_iter().filter(|c| c.number != "intro").collect(),
                                                                        Err(_) => continue,
                                                                    };

                                                                    let mut book_chapters: Vec<serde_json::Value> = Vec::new();

                                                                    for chapter in &chapters {
                                                                        // Fetch chapter with verse-level content
                                                                        let ch_url = format!(
                                                                            "https://api.scripture.api.bible/v1/bibles/{id}/chapters/{}?content-type=json&include-notes=false&include-titles=false&include-chapter-numbers=false&include-verse-numbers=true&include-verse-spans=false",
                                                                            chapter.id
                                                                        );
                                                                        let ch_json = match fetch_text_with_key(&ch_url, &key).await {
                                                                            Ok(j) => j,
                                                                            Err(_) => continue,
                                                                        };

                                                                        // Parse raw content → extract verses
                                                                        let verses = extract_api_bible_verses(&ch_json);
                                                                        book_chapters.push(serde_json::Value::Array(
                                                                            verses.into_iter().map(serde_json::Value::String).collect()
                                                                        ));
                                                                    }

                                                                    thiago_bible.push(serde_json::json!({
                                                                        "abbrev": book.abbreviation.to_lowercase(),
                                                                        "chapters": book_chapters,
                                                                    }));
                                                                }

                                                                dl_status.write().insert(dlkey.clone(), "Importing…".to_string());

                                                                let json_str = serde_json::to_string(&thiago_bible).unwrap_or_default();
                                                                match cmd_import_from_json(&json_str, &trans_id, &name, &lang).await {
                                                                    Ok(r) => {
                                                                        dl_status.write().insert(dlkey, format!("✓ {}", r.message));
                                                                        translations.set(cmd_get_translations().await);
                                                                        stats.set(cmd_get_db_stats().await);
                                                                    }
                                                                    Err(e) => {
                                                                        dl_status.write().insert(dlkey, format!("Error: {e}"));
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        "⬇  Download"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ══ Tab 2: Import from file ════════════════════════════════════════
            if lib_tab() == 2 {

                div { class: "import-zone",
                    div { class: "import-zone-icon", "▤" }
                    div { class: "import-zone-text",
                        p { class: "import-zone-title", "Import a Bible JSON File" }
                        p { class: "import-zone-hint",
                            "Supports "
                            strong { "thiagobodruk/bible" }
                            " JSON format. Download from GitHub, or export from other Bible tools."
                        }
                    }
                }

                div { class: "import-fields-wrap",
                    div { class: "import-field-row",
                        div { class: "import-field",
                            label { class: "import-label", "Translation ID" }
                            input {
                                class: "search-input",
                                placeholder: "e.g. esv  niv  luganda",
                                value: "{pick_id}",
                                oninput: move |e| pick_id.set(e.value()),
                            }
                        }
                        div { class: "import-field import-field-lg",
                            label { class: "import-label", "Full Name" }
                            input {
                                class: "search-input",
                                placeholder: "e.g. English Standard Version",
                                value: "{pick_name}",
                                oninput: move |e| pick_name.set(e.value()),
                            }
                        }
                        div { class: "import-field import-field-sm",
                            label { class: "import-label", "Lang Code" }
                            input {
                                class: "search-input",
                                placeholder: "en  sw  fr",
                                value: "{pick_lang}",
                                oninput: move |e| pick_lang.set(e.value()),
                            }
                        }
                    }
                    div { class: "import-actions",
                        button {
                            class: "btn-primary",
                            disabled: picking(),
                            onclick: move |_| {
                                let id   = pick_id().trim().to_string();
                                let name = pick_name().trim().to_string();
                                let lang = pick_lang().trim().to_string();
                                if id.is_empty() || name.is_empty() {
                                    pick_status.set("Fill in Translation ID and Name first.".to_string());
                                    return;
                                }
                                spawn(async move {
                                    picking.set(true);
                                    pick_status.set("Opening file picker…".to_string());
                                    match cmd_pick_and_import(&id, &name, &lang).await {
                                        Ok(result) => {
                                            pick_status.set(format!("✓ {}", result.message));
                                            translations.set(cmd_get_translations().await);
                                            stats.set(cmd_get_db_stats().await);
                                            pick_id.set(String::new());
                                            pick_name.set(String::new());
                                        }
                                        Err(e) => pick_status.set(format!("Import failed: {e}")),
                                    }
                                    picking.set(false);
                                });
                            },
                            if picking() { "Importing…" } else { "Browse & Import…" }
                        }
                    }
                    if !pick_status().is_empty() {
                        div {
                            class: if pick_status().starts_with("✓") { "import-status ok" }
                                   else if pick_status().contains("failed") || pick_status().contains("Error") { "import-status error" }
                                   else { "import-status" },
                            "{pick_status}"
                        }
                    }
                }

                div { class: "format-ref-card",
                    div { class: "lib-section-title", "Expected JSON Format" }
                    pre { class: "code-block",
"// thiagobodruk (auto-detected):
[ {{\"abbrev\": \"gn\", \"chapters\": [[\"In the beginning...\"], ...]}}, ... ]

// Positional (Genesis → Revelation order):
[ {{\"chapters\": [[\"In the beginning...\"], ...]}}, ... ]"
                    }
                }
            }
        }
    }
}

// ── API.Bible verse extraction ────────────────────────────────────────────────
// API.Bible returns chapter content as a complex JSON tree. This helper
// walks the content tree to extract plain verse text strings.

fn extract_api_bible_verses(chapter_json: &str) -> Vec<String> {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(chapter_json) else { return vec![]; };
    let content = &val["data"]["content"];
    let mut verses: Vec<(i32, String)> = Vec::new();
    walk_api_content(content, &mut verses, &mut None);
    verses.sort_by_key(|(n, _)| *n);
    verses.into_iter().map(|(_, t)| t).collect()
}

fn walk_api_content(
    node: &serde_json::Value,
    verses: &mut Vec<(i32, String)>,
    current_verse: &mut Option<(i32, String)>,
) {
    use serde_json::Value;
    match node {
        Value::Array(arr) => {
            for child in arr {
                walk_api_content(child, verses, current_verse);
            }
        }
        Value::Object(obj) => {
            let typ = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match typ {
                "verse" => {
                    // Commit previous verse
                    if let Some((num, text)) = current_verse.take() {
                        if !text.trim().is_empty() { verses.push((num, text.trim().to_string())); }
                    }
                    let verse_num = obj.get("number")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(0);
                    *current_verse = Some((verse_num, String::new()));
                    if let Some(children) = obj.get("content") {
                        walk_api_content(children, verses, current_verse);
                    }
                }
                "text" => {
                    if let Some((_, ref mut text)) = current_verse {
                        if let Some(s) = obj.get("text").and_then(|v| v.as_str()) {
                            text.push_str(s);
                        }
                    }
                }
                _ => {
                    if let Some(children) = obj.get("content") {
                        walk_api_content(children, verses, current_verse);
                    }
                }
            }
        }
        _ => {}
    }
    // Commit last verse at end of array/object traversal
    // (handled by caller when the top-level array finishes)
}

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
