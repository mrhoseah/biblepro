#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::commands::{
    cmd_list_monitors, cmd_get_outputs, cmd_add_ndi_output, cmd_add_display_output,
    cmd_remove_output, cmd_toggle_output,
    MonitorInfo, OutputInfo, OutputKind,
};

// ── OutputPanel — ProPresenter-style output management ────────────────────────
//
// Compact mode: status pill + active output names (always visible in Present left)
// Expanded mode: visual monitor tiles + full NDI management

#[component]
pub fn OutputPanel() -> Element {
    let mut outputs:  Signal<Vec<OutputInfo>>  = use_signal(Vec::new);
    let mut monitors: Signal<Vec<MonitorInfo>> = use_signal(Vec::new);
    let mut expanded: Signal<bool>             = use_signal(|| false);
    let mut error:    Signal<String>           = use_signal(String::new);

    // NDI form state
    let mut ndi_label:  Signal<String> = use_signal(|| "Main Screen".to_string());
    let mut ndi_source: Signal<String> = use_signal(|| "BiblePro".to_string());
    let mut show_ndi_form: Signal<bool> = use_signal(|| false);

    // Load on mount
    use_effect(move || {
        spawn(async move {
            outputs.set(cmd_get_outputs().await);
            if let Ok(m) = cmd_list_monitors().await { monitors.set(m); }
        });
    });

    // Derived: display outputs mapped by monitor index
    let display_map: Vec<(usize, String, bool)> = outputs().iter()
        .filter_map(|o| {
            if let OutputKind::Display { monitor_index, .. } = &o.kind {
                Some((*monitor_index, o.id.clone(), o.enabled))
            } else { None }
        })
        .collect();

    let ndi_outputs: Vec<OutputInfo> = outputs().into_iter()
        .filter(|o| matches!(o.kind, OutputKind::Ndi { .. }))
        .collect();

    let active_count = display_map.iter().filter(|(_,_,en)| *en).count()
        + ndi_outputs.iter().filter(|o| o.enabled).count();
    let total_count  = display_map.len() + ndi_outputs.len();

    // Active output labels for compact view (e.g. "Display 2", "BiblePro")
    let active_labels: Vec<String> = {
        let mut v: Vec<String> = vec![];
        for (midx, _id, en) in &display_map {
            if *en {
                let mon = monitors().into_iter().find(|m| m.index == *midx);
                v.push(mon.map(|m| m.name.clone()).unwrap_or_else(|| format!("Display {}", midx + 1)));
            }
        }
        for o in &ndi_outputs {
            if o.enabled {
                if let OutputKind::Ndi { source_name } = &o.kind {
                    v.push(format!("NDI: {source_name}"));
                }
            }
        }
        v
    };

    rsx! {
        div { class: "op-panel",

            // ── Status bar (always visible) ───────────────────────────────────
            div { class: "op-status-bar",
                // Connection dot
                div {
                    class: if active_count > 0 { "op-conn-dot active" } else { "op-conn-dot" },
                    title: if active_count > 0 { "Outputs live" } else { "No outputs active" },
                }
                div { class: "op-status-text",
                    if active_count == 0 && total_count == 0 {
                        span { class: "op-no-output", "No outputs — add a screen or NDI" }
                    } else if active_count == 0 {
                        span { class: "op-standby", "Outputs standby" }
                    } else {
                        span { class: "op-live-label", "{active_count} active" }
                        for label in active_labels {
                            span { class: "op-active-chip", "{label}" }
                        }
                    }
                }
                button {
                    class: if expanded() { "op-manage-btn active" } else { "op-manage-btn" },
                    title: if expanded() { "Collapse output panel" } else { "Manage outputs" },
                    onclick: move |_| {
                        expanded.set(!expanded());
                        if expanded() {
                            spawn(async move {
                                outputs.set(cmd_get_outputs().await);
                                if let Ok(m) = cmd_list_monitors().await { monitors.set(m); }
                            });
                        }
                    },
                    if expanded() { "▲" } else { "Screens" }
                }
            }

            // ── Expanded panel ────────────────────────────────────────────────
            if expanded() {

                // ── Displays section ──────────────────────────────────────────
                div { class: "op-section-hdr",
                    span { class: "op-section-title", "Displays" }
                    button {
                        class: "op-icon-btn",
                        title: "Re-scan displays",
                        onclick: move |_| {
                            spawn(async move {
                                match cmd_list_monitors().await {
                                    Ok(m) => { monitors.set(m); error.set(String::new()); }
                                    Err(e) => error.set(e),
                                }
                            });
                        },
                        "↻"
                    }
                }

                if monitors().is_empty() {
                    div { class: "op-empty-hint",
                        "No displays detected. Connect a monitor and press ↻"
                    }
                }

                // Monitor tile grid
                div { class: "op-monitor-grid",
                    for m in monitors() {
                        {
                            let assigned = display_map.iter()
                                .find(|(idx, _, _)| *idx == m.index)
                                .map(|(_, id, en)| (id.clone(), *en));
                            let mon = m.clone();
                            let mon2 = m.clone();
                            let is_live = assigned.as_ref().map(|(_,en)| *en).unwrap_or(false);
                            let is_assigned = assigned.is_some();
                            let tile_class = if is_live {
                                "mon-tile live"
                            } else if is_assigned {
                                "mon-tile assigned"
                            } else {
                                "mon-tile"
                            };

                            rsx! {
                                div { class: "mon-tile-wrap",
                                    // Visual screen tile (aspect-ratio correct)
                                    div { class: "{tile_class}",
                                        if mon.is_primary {
                                            div { class: "mon-primary-badge", "Primary" }
                                        }
                                        if is_live {
                                            div { class: "mon-live-badge", "● LIVE" }
                                        }
                                        // Screen icon
                                        div { class: "mon-screen-art",
                                            div { class: "mon-screen-body",
                                                if is_live {
                                                    div { class: "mon-screen-fill" }
                                                }
                                            }
                                            div { class: "mon-screen-stand" }
                                            div { class: "mon-screen-base" }
                                        }
                                    }
                                    // Label row
                                    div { class: "mon-label-row",
                                        div { class: "mon-label",
                                            if mon.is_primary { "Primary  " } else { "" }
                                            "{mon.name}"
                                        }
                                        div { class: "mon-res", "{mon.width}×{mon.height}" }
                                    }
                                    // Action button
                                    if let Some((id, _)) = assigned.clone() {
                                        {
                                            let id_close  = id.clone();
                                            rsx! {
                                                button {
                                                    class: if is_live { "mon-btn mon-btn-close" } else { "mon-btn mon-btn-inactive" },
                                                    title: "Close output window",
                                                    onclick: move |_| {
                                                        let id = id_close.clone();
                                                        spawn(async move {
                                                            cmd_remove_output(&id).await.ok();
                                                            outputs.set(cmd_get_outputs().await);
                                                        });
                                                    },
                                                    "✕ Close"
                                                }
                                            }
                                        }
                                    } else {
                                        button {
                                            class: "mon-btn mon-btn-project",
                                            title: "Open output on this display",
                                            onclick: move |_| {
                                                let m3 = mon2.clone();
                                                spawn(async move {
                                                    let label = if m3.is_primary {
                                                        format!("Display {} (Primary)", m3.index + 1)
                                                    } else {
                                                        format!("Display {}", m3.index + 1)
                                                    };
                                                    match cmd_add_display_output(
                                                        &label, m3.index, &m3.name,
                                                        m3.x, m3.y, m3.width, m3.height,
                                                    ).await {
                                                        Ok(_)  => { outputs.set(cmd_get_outputs().await); }
                                                        Err(e) => error.set(e),
                                                    }
                                                });
                                            },
                                            "▶ Project"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── NDI section ───────────────────────────────────────────────
                div { class: "op-section-hdr",
                    span { class: "op-section-title", "NDI Outputs" }
                    button {
                        class: if show_ndi_form() { "op-icon-btn active" } else { "op-icon-btn" },
                        title: "Add NDI output",
                        onclick: move |_| show_ndi_form.set(!show_ndi_form()),
                        if show_ndi_form() { "✕" } else { "+" }
                    }
                }

                // Active NDI outputs list
                if ndi_outputs.is_empty() && !show_ndi_form() {
                    div { class: "op-empty-hint",
                        "Add an NDI source to send verse slides over the network to OBS, vMix, ATEM, or ProPresenter."
                    }
                }

                div { class: "op-ndi-list",
                    for output in ndi_outputs {
                        {
                            let id_toggle = output.id.clone();
                            let id_remove = output.id.clone();
                            let source = if let OutputKind::Ndi { source_name } = &output.kind {
                                source_name.clone()
                            } else { String::new() };
                            let source_hint = source.clone();
                            rsx! {
                                div { class: "op-ndi-row",
                                    // Status dot
                                    div {
                                        class: if output.enabled { "op-ndi-dot on" } else { "op-ndi-dot off" },
                                        title: if output.enabled { "Active" } else { "Inactive" },
                                    }
                                    // Labels
                                    div { class: "op-ndi-info",
                                        span { class: "op-ndi-label", "{output.label}" }
                                        span { class: "op-ndi-source", title: "NDI source name visible to other apps", "NDI · {source_hint}" }
                                    }
                                    // Toggle
                                    button {
                                        class: if output.enabled { "op-toggle on" } else { "op-toggle off" },
                                        title: if output.enabled { "Disable" } else { "Enable" },
                                        onclick: move |_| {
                                            let id = id_toggle.clone();
                                            spawn(async move {
                                                cmd_toggle_output(&id).await.ok();
                                                outputs.set(cmd_get_outputs().await);
                                            });
                                        },
                                        if output.enabled { "ON" } else { "OFF" }
                                    }
                                    // Remove
                                    button {
                                        class: "op-remove-btn",
                                        title: "Remove this NDI output",
                                        onclick: move |_| {
                                            let id = id_remove.clone();
                                            spawn(async move {
                                                cmd_remove_output(&id).await.ok();
                                                outputs.set(cmd_get_outputs().await);
                                            });
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                }

                // Add NDI form
                if show_ndi_form() {
                    div { class: "op-ndi-form",
                        div { class: "op-form-row",
                            div { class: "op-form-field",
                                label { class: "op-form-label", "Label" }
                                input {
                                    class: "search-input",
                                    placeholder: "e.g. Main Screen",
                                    value: "{ndi_label}",
                                    oninput: move |e| ndi_label.set(e.value()),
                                }
                            }
                            div { class: "op-form-field",
                                label { class: "op-form-label",
                                    "NDI Source Name"
                                    span { class: "op-form-hint", " — visible to OBS, vMix, etc." }
                                }
                                input {
                                    class: "search-input",
                                    placeholder: "e.g. BiblePro",
                                    value: "{ndi_source}",
                                    oninput: move |e| ndi_source.set(e.value()),
                                }
                            }
                        }
                        button {
                            class: "btn-primary",
                            onclick: move |_| {
                                let label  = ndi_label();
                                let source = ndi_source();
                                if label.trim().is_empty() || source.trim().is_empty() { return; }
                                spawn(async move {
                                    match cmd_add_ndi_output(&label, &source).await {
                                        Ok(_)  => {
                                            outputs.set(cmd_get_outputs().await);
                                            show_ndi_form.set(false);
                                            error.set(String::new());
                                        }
                                        Err(e) => error.set(e),
                                    }
                                });
                            },
                            "Start NDI Output"
                        }
                    }
                }

                // Error display
                if !error().is_empty() {
                    div { class: "op-error", "{error}" }
                }

                // NDI network tip
                div { class: "op-ndi-tip",
                    span { class: "op-tip-icon", "ⓘ" }
                    span { "NDI outputs are visible to any software on your local network. In OBS: Sources → NDI Source → enter the source name above." }
                }
            }
        }
    }
}
