#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::commands::*;
use crate::plans::{PremiumTeaser, FreeBadge};

// ── main Study hub ────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum StudyTab { Bookmarks, Notes, Sets, Plans, Ai }

#[component]
pub fn StudyView() -> Element {
    let mut tab = use_signal(|| StudyTab::Bookmarks);

    rsx! {
        div { class: "study-root",
            // Tab bar
            div { class: "study-tabs",
                StudyTabBtn { label: "Bookmarks",  active: tab() == StudyTab::Bookmarks, onclick: move |_| tab.set(StudyTab::Bookmarks) }
                StudyTabBtn { label: "Notes",      active: tab() == StudyTab::Notes,     onclick: move |_| tab.set(StudyTab::Notes) }
                StudyTabBtn { label: "Study Sets", active: tab() == StudyTab::Sets,      onclick: move |_| tab.set(StudyTab::Sets) }
                StudyTabBtn { label: "Plans",      active: tab() == StudyTab::Plans,     onclick: move |_| tab.set(StudyTab::Plans) }
                StudyTabBtn { label: "AI Tools",   active: tab() == StudyTab::Ai,        onclick: move |_| tab.set(StudyTab::Ai) }
            }
            div { class: "study-content",
                match tab() {
                    StudyTab::Bookmarks => rsx! { BookmarksPanel {} },
                    StudyTab::Notes     => rsx! { NotesPanel {} },
                    StudyTab::Sets      => rsx! { SetsPanel {} },
                    StudyTab::Plans     => rsx! { PlansPanel {} },
                    StudyTab::Ai        => rsx! { AiToolsPanel {} },
                }
            }
        }
    }
}

#[component]
fn StudyTabBtn(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: if active { "study-tab active" } else { "study-tab" },
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

// ── bookmarks ─────────────────────────────────────────────────────────────────

#[component]
fn BookmarksPanel() -> Element {
    let mut bookmarks = use_signal(|| Vec::<Bookmark>::new());
    let mut loading   = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            bookmarks.set(cmd_get_bookmarks().await);
            loading.set(false);
        });
    });

    let do_delete = move |id: i64| {
        spawn(async move {
            cmd_delete_bookmark(id).await;
            bookmarks.set(cmd_get_bookmarks().await);
        });
    };

    rsx! {
        div { class: "study-panel",
            div { class: "study-panel-hdr",
                h3 { "Bookmarks" }
                span { class: "study-count", "{bookmarks.read().len()} saved" }
            }

            if loading() {
                div { class: "study-loading", "Loading…" }
            } else if bookmarks.read().is_empty() {
                div { class: "study-empty",
                    div { class: "study-empty-icon", "🔖" }
                    p { "No bookmarks yet." }
                    p { class: "study-empty-hint", "In the Reader, click any verse number to open the study toolbar and save bookmarks." }
                }
            } else {
                div { class: "bookmark-list",
                    for bm in bookmarks.read().clone() {
                        div { class: "bookmark-card",
                            div { class: "bm-header",
                                div { class: "bm-ref", "{bm.book_name} {bm.chapter}:{bm.verse}" }
                                button {
                                    class: "bm-delete",
                                    title: "Remove bookmark",
                                    onclick: move |_| do_delete(bm.id),
                                    "✕"
                                }
                            }
                            p { class: "bm-text", "{bm.verse_text}" }
                        }
                    }
                }
            }
        }
    }
}

// ── notes ─────────────────────────────────────────────────────────────────────

#[component]
fn NotesPanel() -> Element {
    let mut notes    = use_signal(|| Vec::<Note>::new());
    let mut books    = use_signal(|| Vec::<crate::commands::Book>::new());
    let mut search   = use_signal(|| String::new());
    let mut loading  = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            notes.set(cmd_get_all_notes().await);
            books.set(crate::commands::cmd_get_books().await);
            loading.set(false);
        });
    });

    let q = search();
    let filtered: Vec<Note> = notes.read().iter()
        .filter(|n| q.is_empty() || n.body.to_lowercase().contains(&q.to_lowercase()))
        .cloned()
        .collect();

    rsx! {
        div { class: "study-panel",
            div { class: "study-panel-hdr",
                h3 { "My Notes" }
                span { class: "study-count", "{notes.read().len()} note(s)" }
            }
            input {
                class: "search-input",
                placeholder: "Search notes…",
                value: "{search}",
                oninput: move |e| search.set(e.value()),
            }

            if loading() {
                div { class: "study-loading", "Loading…" }
            } else if notes.read().is_empty() {
                div { class: "study-empty",
                    div { class: "study-empty-icon", "📝" }
                    p { "No notes yet." }
                    p { class: "study-empty-hint", "Click a verse in the Reader to open the study toolbar and add notes." }
                }
            } else if filtered.is_empty() {
                div { class: "study-empty",
                    p { "No notes match \"{search}\"" }
                }
            } else {
                div { class: "notes-list",
                    for note in filtered {
                        {
                            let book_name = books.read().iter()
                                .find(|b| b.id == note.book_id)
                                .map(|b| b.name.clone())
                                .unwrap_or_else(|| format!("Book {}", note.book_id));
                            rsx! {
                                div { class: "note-card",
                                    div { class: "note-ref", "{book_name} {note.chapter}:{note.verse}" }
                                    p { class: "note-body", "{note.body}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── study sets ────────────────────────────────────────────────────────────────

#[component]
fn SetsPanel() -> Element {
    let mut sets         = use_signal(|| Vec::<StudySet>::new());
    let mut active_set   = use_signal(|| Option::<i64>::None);
    let mut set_verses   = use_signal(|| Vec::<SetVerse>::new());
    let mut new_name     = use_signal(|| String::new());
    let mut new_desc     = use_signal(|| String::new());
    let mut export_text  = use_signal(|| String::new());
    let mut status       = use_signal(|| String::new());

    use_effect(move || {
        spawn(async move { sets.set(cmd_get_study_sets().await); });
    });

    let mut load_set = move |id: i64| {
        active_set.set(Some(id));
        spawn(async move { set_verses.set(cmd_get_set_verses(id).await); });
    };

    let do_create = move |_| {
        let name = new_name().trim().to_string();
        if name.is_empty() { return; }
        let desc = new_desc().trim().to_string();
        spawn(async move {
            cmd_create_study_set(&name, &desc).await;
            sets.set(cmd_get_study_sets().await);
            new_name.set(String::new());
            new_desc.set(String::new());
        });
    };

    let do_delete_set = move |id: i64| {
        spawn(async move {
            cmd_delete_study_set(id).await;
            sets.set(cmd_get_study_sets().await);
            if active_set() == Some(id) { active_set.set(None); }
        });
    };

    let do_export = move |id: i64| {
        spawn(async move {
            if let Some(json) = cmd_export_study_set(id).await {
                export_text.set(json);
                status.set("JSON ready — copy it to share this study set.".into());
            }
        });
    };

    rsx! {
        div { class: "study-panel sets-panel",
            // Left: set list
            div { class: "sets-list-col",
                div { class: "study-panel-hdr",
                    h3 { "Study Sets" }
                }

                // Create new
                div { class: "set-create-form",
                    input {
                        class: "search-input",
                        placeholder: "Set name…",
                        value: "{new_name}",
                        oninput: move |e| new_name.set(e.value()),
                    }
                    input {
                        class: "search-input",
                        placeholder: "Description (optional)",
                        value: "{new_desc}",
                        oninput: move |e| new_desc.set(e.value()),
                    }
                    button { class: "btn-primary", onclick: do_create, "Create Set" }
                }

                if sets.read().is_empty() {
                    div { class: "study-empty",
                        div { class: "study-empty-icon", "📚" }
                        p { "No study sets yet." }
                        p { class: "study-empty-hint", "Create a set, then add verses from the Reader." }
                    }
                } else {
                    div { class: "set-cards",
                        for s in sets.read().clone() {
                            div {
                                class: if active_set() == Some(s.id) { "set-card active" } else { "set-card" },
                                onclick: move |_| load_set(s.id),
                                div { class: "set-card-name", "{s.name}" }
                                if !s.description.is_empty() {
                                    div { class: "set-card-desc", "{s.description}" }
                                }
                                div { class: "set-card-meta", "{s.verse_count} verse(s)" }
                                div { class: "set-card-actions",
                                    button {
                                        class: "btn-ghost sm-btn",
                                        onclick: move |e| { e.stop_propagation(); do_export(s.id); },
                                        "Export JSON"
                                    }
                                    button {
                                        class: "btn-danger sm-btn",
                                        onclick: move |e| { e.stop_propagation(); do_delete_set(s.id); },
                                        "Delete"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Right: verses in active set
            div { class: "set-verses-col",
                if let Some(_id) = active_set() {
                    div { class: "study-panel-hdr",
                        h3 { "Verses" }
                        span { class: "study-count", "{set_verses.read().len()} verse(s)" }
                    }
                    if set_verses.read().is_empty() {
                        div { class: "study-empty",
                            p { "No verses in this set yet." }
                            p { class: "study-empty-hint", "In the Reader, select a verse and choose 'Add to Set'." }
                        }
                    } else {
                        div { class: "set-verse-list",
                            for sv in set_verses.read().clone() {
                                div { class: "set-verse-card",
                                    div { class: "sv-ref", "{sv.book_name} {sv.chapter}:{sv.verse}" }
                                    p { class: "sv-text", "{sv.verse_text}" }
                                    if !sv.note.is_empty() {
                                        div { class: "sv-note", "📝 {sv.note}" }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "study-empty",
                        p { "Select a study set to view its verses." }
                    }
                }

                if !export_text().is_empty() {
                    div { class: "export-box",
                        div { class: "export-label", "Exported JSON (share this)" }
                        textarea {
                            class: "export-textarea",
                            readonly: true,
                            rows: "8",
                            value: "{export_text}",
                        }
                    }
                }
                if !status().is_empty() {
                    div { class: "status-chip", "{status}" }
                }
            }
        }
    }
}

// ── reading plans ─────────────────────────────────────────────────────────────

#[component]
fn PlansPanel() -> Element {
    let mut plans       = use_signal(|| Vec::<ReadingPlan>::new());
    let mut active_plan = use_signal(|| Option::<String>::None);
    let mut plan_days   = use_signal(|| Vec::<PlanDay>::new());

    use_effect(move || {
        spawn(async move { plans.set(cmd_get_reading_plans().await); });
    });

    let mut open_plan = move |id: String| {
        let id2 = id.clone();
        active_plan.set(Some(id));
        spawn(async move { plan_days.set(cmd_get_plan_days(&id2).await); });
    };

    let toggle_day = move |plan_id: String, day: i32, done: bool| {
        let plan_id2 = plan_id.clone();
        spawn(async move {
            cmd_mark_plan_day(&plan_id2, day, !done).await;
            plan_days.set(cmd_get_plan_days(&plan_id2).await);
            plans.set(cmd_get_reading_plans().await);
        });
    };

    rsx! {
        div { class: "study-panel plans-panel",
            div { class: "plans-list-col",
                div { class: "study-panel-hdr",
                    h3 { "Reading Plans" }
                }
                div { class: "plan-cards",
                    for plan in plans.read().clone() {
                        div {
                            class: if active_plan().as_deref() == Some(&plan.id) { "plan-card active" } else { "plan-card" },
                            onclick: move |_| open_plan(plan.id.clone()),
                            div { class: "plan-name", "{plan.name}" }
                            p { class: "plan-desc", "{plan.description}" }
                            div { class: "plan-progress-row",
                                div { class: "plan-progress-bar",
                                    div {
                                        class: "plan-progress-fill",
                                        style: "width:{(plan.days_done as f32 / plan.total_days as f32 * 100.0) as i32}%;",
                                    }
                                }
                                span { class: "plan-days-label",
                                    "{plan.days_done}/{plan.total_days} days"
                                }
                            }
                        }
                    }
                }
            }

            div { class: "plan-days-col",
                if active_plan().is_some() {
                    div { class: "study-panel-hdr",
                        h3 {
                            if let Some(ref id) = active_plan() {
                                if let Some(p) = plans.read().iter().find(|p| &p.id == id) {
                                    "{p.name}"
                                } else { "Reading Plan" }
                            } else { "Reading Plan" }
                        }
                    }
                    div { class: "day-list",
                        for day in plan_days.read().clone() {
                            {
                                let plan_id = active_plan().unwrap_or_default();
                                let day_num = day.day;
                                let is_done = day.completed;
                                rsx! {
                                    div {
                                        class: if day.completed { "day-row done" } else { "day-row" },
                                        button {
                                            class: if day.completed { "day-check checked" } else { "day-check" },
                                            title: if day.completed { "Mark incomplete" } else { "Mark complete" },
                                            onclick: move |_| toggle_day(plan_id.clone(), day_num, is_done),
                                            if day.completed { "✓" } else { "" }
                                        }
                                        div { class: "day-body",
                                            div { class: "day-label", "Day {day.day} — {day.label}" }
                                            div { class: "day-passages",
                                                for p in &day.passages {
                                                    span { class: "passage-chip", "{p.book_name} {p.chapter}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "study-empty",
                        div { class: "study-empty-icon", "📅" }
                        p { "Select a reading plan to see daily readings." }
                        p { class: "study-empty-hint", "Check off each day as you read. Progress is saved automatically." }
                    }
                }
            }
        }
    }
}

// ── AI tools panel (premium teaser) ──────────────────────────────────────────

#[component]
fn AiToolsPanel() -> Element {
    rsx! {
        div { class: "study-panel ai-panel",
            div { class: "study-panel-hdr",
                h3 { "AI Study Tools" }
                span { class: "plan-badge badge-ai", "✦ AI Plan" }
            }
            div { class: "ai-panel-body",
                div { class: "ai-panel-intro",
                    p {
                        "Powerful AI tools to study deeper, prepare faster, and understand Scripture in context. "
                        "Available in the "
                        strong { "Church AI" }
                        " plan — currently in development."
                    }
                }
                div { class: "ai-teasers",
                    PremiumTeaser {
                        icon: "🤖",
                        title: "AI Sermon Assistant",
                        desc: "Generate sermon outlines, illustrations, and talking points from any scripture passage.",
                        tier: "AI"
                    }
                    PremiumTeaser {
                        icon: "💬",
                        title: "AI Bible Study Chat",
                        desc: "Ask questions about any passage and get contextual, theologically-grounded answers.",
                        tier: "AI"
                    }
                    PremiumTeaser {
                        icon: "🔎",
                        title: "Semantic Scripture Search",
                        desc: "Search by concept and theme — find verses about forgiveness even without the word.",
                        tier: "AI"
                    }
                    PremiumTeaser {
                        icon: "📜",
                        title: "Original Language Insights",
                        desc: "Greek and Hebrew word studies with contextual meaning and cross-references.",
                        tier: "AI"
                    }
                    PremiumTeaser {
                        icon: "📋",
                        title: "AI Study Summaries",
                        desc: "Instant chapter summaries, key themes, and study questions for any passage.",
                        tier: "AI"
                    }
                    PremiumTeaser {
                        icon: "🎞️",
                        title: "AI Slide Generation",
                        desc: "Turn a sermon outline into a ready-to-present slide deck in seconds.",
                        tier: "AI"
                    }
                }
                div { class: "ai-panel-cta",
                    a {
                        class: "btn-primary",
                        href: "mailto:hoseahkplgt@gmail.com?subject=BiblePro%20AI%20Plan%20Interest",
                        "Express Interest in AI Plan"
                    }
                    div { class: "ai-free-note",
                        FreeBadge {}
                        span { "Notes, bookmarks, reading plans and study sets are always free." }
                    }
                }
            }
        }
    }
}
