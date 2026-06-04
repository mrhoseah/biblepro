#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::commands::*;

// Split FTS highlight markers «…» into (text, is_match) segments for rendering
fn hl_segments(s: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    let mut rest = s;
    loop {
        match rest.find('\u{ab}') {
            None => { if !rest.is_empty() { out.push((rest.to_string(), false)); } break; }
            Some(i) => {
                if i > 0 { out.push((rest[..i].to_string(), false)); }
                rest = &rest[i + '\u{ab}'.len_utf8()..];
                match rest.find('\u{bb}') {
                    None => { out.push((rest.to_string(), true)); break; }
                    Some(j) => {
                        out.push((rest[..j].to_string(), true));
                        rest = &rest[j + '\u{bb}'.len_utf8()..];
                    }
                }
            }
        }
    }
    out
}

#[component]
pub fn SearchView(books: Signal<Vec<Book>>, translations: Signal<Vec<Translation>>) -> Element {
    let mut sel_trans  = use_signal(|| String::new());
    let mut query      = use_signal(|| String::new());
    let mut testament  = use_signal(|| String::from("all"));
    let mut sel_book   = use_signal(|| 0i32);
    let mut limit      = use_signal(|| 50i32);
    let mut results    = use_signal(|| Vec::<SearchResult>::new());
    let mut searching  = use_signal(|| false);
    let mut jumped     = use_signal(|| Option::<Verse>::None);
    let mut result_msg = use_signal(|| String::new());

    use_effect(move || {
        let list = translations();
        if sel_trans.read().is_empty() {
            if let Some(first) = list.first() {
                sel_trans.set(first.id.clone());
            }
        }
    });

    let do_search = move || {
        let q = query();
        let t = sel_trans();
        if q.trim().is_empty() || t.is_empty() { return; }
        let book_filter = if sel_book() == 0 { None } else { Some(sel_book()) };
        let test_filter: Option<String> = match testament().as_str() {
            "OT" => Some("OT".to_string()),
            "NT" => Some("NT".to_string()),
            _    => None,
        };
        let lim = limit();
        spawn(async move {
            searching.set(true);
            result_msg.set(String::new());
            if q.contains(':') {
                if let Some(v) = cmd_search_by_reference(&t, &q).await {
                    jumped.set(Some(v));
                    results.set(vec![]);
                    searching.set(false);
                    return;
                }
            }
            jumped.set(None);
            let r = cmd_search_verses(&t, &q, Some(lim), book_filter, test_filter.as_deref()).await;
            let count = r.len();
            results.set(r);
            result_msg.set(if count == 0 {
                format!("No results for \"{}\"", q)
            } else {
                format!("{} result{}", count, if count == 1 { "" } else { "s" })
            });
            searching.set(false);
        });
    };
    let do_search2 = do_search.clone();

    let book_list: Vec<Book> = {
        let t = testament();
        books().into_iter().filter(|b| t == "all" || b.testament == t).collect()
    };

    rsx! {
        div { class: "search-view",

            div { class: "search-trans-bar",
                span { class: "search-bar-label", "Translation" }
                div { class: "search-trans-chips",
                    for tr in translations() {
                        {
                            let tid    = tr.id.clone();
                            let active = sel_trans() == tr.id;
                            rsx! {
                                button {
                                    class: if active { "trans-pill active" } else { "trans-pill" },
                                    title: "{tr.name}  ·  {tr.language}",
                                    onclick: move |_| sel_trans.set(tid.clone()),
                                    span { class: "pill-abbr", "{tr.abbreviation}" }
                                }
                            }
                        }
                    }
                    if translations().is_empty() {
                        span { class: "search-bar-label", "No translations — import one in Library" }
                    }
                }
            }

            div { class: "search-input-row",
                input {
                    class: "search-input search-main-input",
                    placeholder: "Search text, phrase, or reference (e.g. John 3:16, grace, love one another)…",
                    value: "{query}",
                    oninput: move |e| query.set(e.value()),
                    onkeydown: move |e| { if e.key() == Key::Enter { do_search2(); } },
                }
                button {
                    class: "btn-primary search-go",
                    disabled: searching(),
                    onclick: move |_| do_search(),
                    if searching() { "…" } else { "Search" }
                }
            }

            div { class: "search-filter-row",
                div { class: "filter-group",
                    span { class: "filter-label", "Testament" }
                    div { class: "seg-ctrl",
                        for (val, lbl) in [("all","All"),("OT","Old"),("NT","New")] {
                            {
                                let v = val.to_string();
                                let is_active = testament() == val;
                                rsx! {
                                    button {
                                        class: if is_active { "seg-btn active" } else { "seg-btn" },
                                        onclick: move |_| { testament.set(v.clone()); sel_book.set(0); },
                                        "{lbl}"
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "filter-group",
                    span { class: "filter-label", "Book" }
                    select {
                        class: "select filter-select",
                        value: "{sel_book}",
                        onchange: move |e| sel_book.set(e.value().parse().unwrap_or(0)),
                        option { value: "0", "All books" }
                        for b in book_list {
                            option { value: "{b.id}", "{b.name}" }
                        }
                    }
                }

                div { class: "filter-group",
                    span { class: "filter-label", "Limit" }
                    select {
                        class: "select filter-select",
                        value: "{limit}",
                        onchange: move |e| limit.set(e.value().parse().unwrap_or(50)),
                        option { value: "25",  "25" }
                        option { value: "50",  "50" }
                        option { value: "100", "100" }
                        option { value: "200", "200" }
                        option { value: "500", "500" }
                    }
                }

                if sel_book() != 0 || testament() != "all" {
                    button {
                        class: "btn-ghost sm-btn",
                        onclick: move |_| { testament.set("all".to_string()); sel_book.set(0); },
                        "✕ Clear filters"
                    }
                }
            }

            div { class: "results-area",
                if let Some(v) = jumped() {
                    div { class: "reference-result",
                        div { class: "ref-badge", "{v.book_name} {v.chapter}:{v.verse}" }
                        p { class: "ref-text", "{v.text}" }
                    }
                } else if searching() {
                    div { class: "search-spinner",
                        div { class: "spinner" }
                        span { "Searching…" }
                    }
                } else if !results().is_empty() {
                    div { class: "result-header",
                        span { class: "result-count-badge", "{result_msg}" }
                        if results().len() as i32 >= limit() {
                            span { class: "result-cap-hint",
                                "Showing first {limit()} — refine your query or increase limit"
                            }
                        }
                    }
                    div { class: "results-list",
                        for r in results() {
                            SearchResultRow { key: "{r.verse.id}", result: r }
                        }
                    }
                } else if !result_msg().is_empty() {
                    div { class: "no-results",
                        div { class: "no-results-icon", "⌕" }
                        p { "{result_msg}" }
                        p { class: "no-results-hint", "Try different words, check spelling, or widen filters." }
                    }
                } else {
                    div { class: "search-welcome",
                        div { class: "search-welcome-icon", "✦" }
                        p { class: "search-welcome-title", "Full-Text Bible Search" }
                        div { class: "search-tips",
                            div { class: "search-tip",
                                span { class: "tip-icon", "⬡" }
                                div {
                                    strong { "Phrase or word" }
                                    p { "grace  ·  love your enemies  ·  fear not" }
                                }
                            }
                            div { class: "search-tip",
                                span { class: "tip-icon", "◈" }
                                div {
                                    strong { "Reference" }
                                    p { "John 3:16  ·  Romans 8:28  ·  Psalm 23:1" }
                                }
                            }
                            div { class: "search-tip",
                                span { class: "tip-icon", "▤" }
                                div {
                                    strong { "Filter by testament or book" }
                                    p { "Narrow results with the filters above" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SearchResultRow(result: SearchResult) -> Element {
    let v    = result.verse;
    let segs = hl_segments(&result.snippet);
    rsx! {
        div { class: "result-row",
            div { class: "result-ref",
                span { class: "result-book", "{v.book_name}" }
                span { class: "result-cv", " {v.chapter}:{v.verse}" }
            }
            p { class: "result-text",
                for (seg, is_match) in segs {
                    if is_match {
                        mark { class: "search-hl", "{seg}" }
                    } else {
                        span { "{seg}" }
                    }
                }
            }
        }
    }
}
