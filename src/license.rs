#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::commands::{
    cmd_activate_license, cmd_deactivate_license, cmd_get_license_status, cmd_refresh_license,
    LicenseStatus, Plan,
};

// ── LicenseView (full page — shown from Plans/About tab) ─────────────────────

#[component]
pub fn LicenseView() -> Element {
    let mut status: Signal<Option<LicenseStatus>> = use_signal(|| None);
    let mut token_input = use_signal(String::new);
    let mut msg = use_signal(String::new);
    let mut busy = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            status.set(cmd_get_license_status().await);
            // Attempt silent refresh in background
            let _ = cmd_refresh_license().await;
            status.set(cmd_get_license_status().await);
        });
    });

    let do_activate = move |_| {
        let t = token_input().trim().to_string();
        if t.is_empty() { msg.set("Paste your license key above.".to_string()); return; }
        spawn(async move {
            busy.set(true);
            msg.set(String::new());
            match cmd_activate_license(&t).await {
                Ok(s)  => { status.set(Some(s)); token_input.set(String::new()); msg.set("License activated.".to_string()); }
                Err(e) => { msg.set(format!("Activation failed: {e}")); }
            }
            busy.set(false);
        });
    };

    let do_deactivate = move |_| {
        spawn(async move {
            busy.set(true);
            let _ = cmd_deactivate_license().await;
            status.set(cmd_get_license_status().await);
            msg.set("License removed from this device.".to_string());
            busy.set(false);
        });
    };

    rsx! {
        div { class: "license-view",
            div { class: "license-header",
                h2 { class: "license-title", "License & Activation" }
                p { class: "license-sub",
                    "Your license key is sent to you by email after purchase at "
                    a { href: "#", class: "lic-link", "zehut.io/biblepro" }
                    "."
                }
            }

            // ── Current status card ───────────────────────────────────────────
            if let Some(s) = status() {
                div { class: "lic-status-card",
                    div { class: "lic-plan-row",
                        span { class: format!("lic-plan-badge {}", s.plan.label().to_lowercase()),
                            "{s.plan.label()} Plan"
                        }
                        if s.is_in_grace {
                            span { class: "lic-grace-badge",
                                "Grace period — {s.grace_days_remaining.unwrap_or(0)} days left"
                            }
                        }
                    }

                    if !s.org.is_empty() {
                        div { class: "lic-org", "{s.org}" }
                    }

                    div { class: "lic-meta",
                        span { class: "lic-meta-item",
                            span { class: "lic-meta-label", "Device ID" }
                            code { class: "lic-device-id", "{s.device_id}" }
                        }
                        if let Some(exp) = s.expires_at {
                            {
                                let date = format_unix_date(exp);
                                rsx! {
                                    span { class: "lic-meta-item",
                                        span { class: "lic-meta-label", "Expires" }
                                        span { "{date}" }
                                    }
                                }
                            }
                        }
                    }

                    if s.plan != Plan::Free {
                        button {
                            class: "btn-ghost sm-btn lic-deactivate",
                            disabled: busy(),
                            onclick: do_deactivate,
                            "Remove from this device"
                        }
                    }
                }
            }

            // ── Activation form ───────────────────────────────────────────────
            div { class: "lic-activate-section",
                h3 { class: "lic-section-title", "Activate a License Key" }
                p { class: "lic-activate-hint",
                    "Paste the license key from your purchase email. Keys are bound to one device. "
                    "To move to a new machine, deactivate here first or visit the Zehut portal."
                }
                textarea {
                    class: "lic-token-input",
                    placeholder: "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9…",
                    rows: "4",
                    value: "{token_input}",
                    oninput: move |e| token_input.set(e.value()),
                }
                button {
                    class: "btn-primary",
                    disabled: busy() || token_input().trim().is_empty(),
                    onclick: do_activate,
                    if busy() { "Verifying…" } else { "Activate" }
                }
                if !msg().is_empty() {
                    div {
                        class: if msg().starts_with("License activated") || msg().starts_with("License removed") {
                            "lic-msg ok"
                        } else {
                            "lic-msg err"
                        },
                        "{msg}"
                    }
                }
            }

            // ── Plan comparison ───────────────────────────────────────────────
            div { class: "lic-plans",
                h3 { class: "lic-section-title", "What's included" }
                div { class: "lic-plan-grid",
                    PlanCard {
                        name: "Free",
                        price: "Always free",
                        features: vec![
                            "Full Bible reader & search",
                            "Highlights, notes, bookmarks",
                            "Reading plans",
                            "NDI output (1080p)",
                            "FullScreen & LowerThird templates",
                            "Solid colour backgrounds",
                        ],
                        is_current: status().map(|s| s.plan == Plan::Free).unwrap_or(true),
                    }
                    PlanCard {
                        name: "Standard",
                        price: "Contact zehut.io",
                        features: vec![
                            "Everything in Free",
                            "Advanced templates (LT Accent, Split, Card)",
                            "Canvas gradient background designer",
                            "4K (3840×2160) NDI output",
                            "14-day offline grace period",
                        ],
                        is_current: status().map(|s| s.plan == Plan::Standard).unwrap_or(false),
                    }
                    PlanCard {
                        name: "Premium",
                        price: "Coming soon",
                        features: vec![
                            "Everything in Standard",
                            "AI verse suggestions",
                            "Cloud setlist sync",
                            "Multiple NDI outputs",
                            "Shared media library",
                        ],
                        is_current: status().map(|s| s.plan == Plan::Premium).unwrap_or(false),
                    }
                }
            }
        }
    }
}

// ── LicenseBanner (inline widget shown at top of Present tab) ─────────────────

#[component]
pub fn LicenseBanner() -> Element {
    let mut status: Signal<Option<LicenseStatus>> = use_signal(|| None);

    use_effect(move || {
        spawn(async move { status.set(cmd_get_license_status().await); });
    });

    let Some(s) = status() else { return rsx! {} };

    // Only show banner if in grace or on Free plan trying to use present features
    if s.plan != Plan::Free && !s.is_in_grace { return rsx! {}; }

    if s.is_in_grace {
        rsx! {
            div { class: "lic-banner grace",
                span { class: "lic-banner-icon", "⚠" }
                span { "License expired — {s.grace_days_remaining.unwrap_or(0)} grace days remaining. Reconnect to renew." }
            }
        }
    } else {
        rsx! {} // Free plan banner only shown when a premium action fails
    }
}

// ── sub-components ────────────────────────────────────────────────────────────

#[component]
fn PlanCard(
    name: &'static str,
    price: &'static str,
    features: Vec<&'static str>,
    is_current: bool,
) -> Element {
    rsx! {
        div { class: if is_current { "lic-plan-card current" } else { "lic-plan-card" },
            if is_current {
                div { class: "lic-current-badge", "Current plan" }
            }
            div { class: "lic-plan-name", "{name}" }
            div { class: "lic-plan-price", "{price}" }
            ul { class: "lic-feature-list",
                for feat in features {
                    li { class: "lic-feature-item",
                        span { class: "lic-feat-check", "✓" }
                        span { "{feat}" }
                    }
                }
            }
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn format_unix_date(unix: i64) -> String {
    // Simple date without external crates — sufficient for display
    let secs = unix as u64;
    let days_since_epoch = secs / 86400;
    // Rough calendar math (good enough for display purposes)
    let years = 1970 + days_since_epoch / 365;
    let remaining_days = days_since_epoch % 365;
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let month_idx = (remaining_days / 30).min(11) as usize;
    let day = remaining_days % 30 + 1;
    format!("{} {} {}", day, months[month_idx], years)
}
