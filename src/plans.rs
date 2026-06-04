#![allow(non_snake_case)]

use dioxus::prelude::*;

// ── shared badge/teaser components (imported by other views) ──────────────────

#[component]
pub fn PremiumBadge(tier: &'static str) -> Element {
    let (cls, label) = match tier {
        "Standard"   => ("badge-standard",   "✦ Standard"),
        "AI"         => ("badge-ai",          "✦ AI"),
        "Enterprise" => ("badge-enterprise",  "✦ Enterprise"),
        _            => ("badge-free",        "Free"),
    };
    rsx! { span { class: "plan-badge {cls}", "{label}" } }
}

#[component]
pub fn FreeBadge() -> Element {
    rsx! { span { class: "plan-badge badge-free", "Free" } }
}

/// A locked-but-graceful teaser for a coming premium feature.
#[component]
pub fn PremiumTeaser(
    icon: &'static str,
    title: &'static str,
    desc: &'static str,
    tier: &'static str,
) -> Element {
    rsx! {
        div { class: "premium-teaser",
            span { class: "pt-icon", "{icon}" }
            div { class: "pt-body",
                div { class: "pt-title",
                    "{title}"
                    PremiumBadge { tier }
                }
                p { class: "pt-desc", "{desc}" }
            }
            a {
                class: "btn-try",
                href: "mailto:hoseahkplgt@gmail.com?subject=BiblePro%20Premium%20Interest&body=I%27m%20interested%20in%20the%20{tier}%20plan.",
                "Try Premium"
            }
        }
    }
}

// ── plan tier data ────────────────────────────────────────────────────────────

struct PlanDef {
    name:     &'static str,
    tagline:  &'static str,
    badge:    &'static str,
    cta:      &'static str,
    features: &'static [&'static str],
    highlight: bool,
}

const PLANS: &[PlanDef] = &[
    PlanDef {
        name:    "Free",
        tagline: "Essential worship tools — no credit card, forever.",
        badge:   "Free",
        cta:     "Always Free",
        features: &[
            "Bible projection & verse display",
            "Multiple translations",
            "Full-screen & lower-third layouts",
            "Full-text scripture search",
            "Reference lookup (John 3:16 style)",
            "Highlights, notes & bookmarks",
            "Reading plans & study sets",
            "Offline mode — everything local",
            "NDI output (single source, 1080p)",
            "Basic setlist & service flow",
        ],
        highlight: false,
    },
    PlanDef {
        name:    "Church Standard",
        tagline: "Advanced presentation for growing ministries.",
        badge:   "Standard",
        cta:     "Contact for pricing",
        features: &[
            "Everything in Free",
            "All 6 layout templates",
            "Cinematic motion packs & animations",
            "Advanced lower-third graphics",
            "Multiple NDI outputs",
            "4K resolution output",
            "Chroma key & alpha channel support",
            "Livestream overlay graphics",
            "Advanced background themes",
            "Priority support",
        ],
        highlight: true,
    },
    PlanDef {
        name:    "Church AI",
        tagline: "Intelligence and productivity for media teams.",
        badge:   "AI",
        cta:     "Contact for pricing",
        features: &[
            "Everything in Standard",
            "AI Sermon Assistant",
            "AI Bible Study Chat",
            "Semantic scripture search",
            "AI slide generation",
            "Contextual Bible insights",
            "Original language tools (Greek/Hebrew)",
            "AI outline & summary generator",
            "AI translation assistance",
            "Cloud-powered AI processing",
        ],
        highlight: false,
    },
    PlanDef {
        name:    "Enterprise",
        tagline: "Centralized control for multi-campus organizations.",
        badge:   "Enterprise",
        cta:     "Contact us",
        features: &[
            "Everything in Church AI",
            "Multi-campus management",
            "Team collaboration & approvals",
            "Centralized media library",
            "Role-based access control",
            "Usage analytics & reporting",
            "API access",
            "Dedicated onboarding & training",
            "SLA & priority support",
            "Custom integrations",
        ],
        highlight: false,
    },
];

// ── plans view ────────────────────────────────────────────────────────────────

#[component]
pub fn PlansView() -> Element {
    rsx! {
        div { class: "plans-view",
            div { class: "plans-inner",

                // Header
                div { class: "plans-hero",
                    h1 { class: "plans-title", "Simple, Ministry-Focused Pricing" }
                    p { class: "plans-sub",
                        "Keep the essentials free. Pay for what makes your team faster, smarter, and more effective."
                    }
                }

                // Strategy callout
                div { class: "plans-callout",
                    span { class: "callout-icon", "🏛️" }
                    p {
                        strong { "Our commitment: " }
                        "Basic Bible projection, offline functionality, and essential service operations are "
                        strong { "always free." }
                        " Premium unlocks advanced productivity — not basic ministry."
                    }
                }

                // Plan cards
                div { class: "plan-cards-grid",
                    for plan in PLANS {
                        div {
                            class: if plan.highlight { "plan-card highlighted" } else { "plan-card" },
                            if plan.highlight {
                                div { class: "plan-card-banner", "Most Popular" }
                            }
                            div { class: "plan-card-header",
                                div { class: "plan-name-row",
                                    h2 { class: "plan-card-name", "{plan.name}" }
                                    PremiumBadge { tier: plan.badge }
                                }
                                p { class: "plan-tagline", "{plan.tagline}" }
                                div { class: "plan-cta-row",
                                    if plan.badge == "Free" {
                                        span { class: "plan-price-free", "{plan.cta}" }
                                    } else {
                                        a {
                                            class: "btn-plan-cta",
                                            href: "mailto:hoseahkplgt@gmail.com?subject=BiblePro%20{plan.name}%20Inquiry",
                                            "{plan.cta}"
                                        }
                                    }
                                }
                            }
                            div { class: "plan-features",
                                for feat in plan.features {
                                    div { class: "plan-feat",
                                        span { class: "feat-check", "✓" }
                                        span { "{feat}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Premium strategy section
                div { class: "plans-strategy",
                    h3 { class: "strategy-title", "Why This Model?" }
                    div { class: "strategy-grid",
                        StrategyCard {
                            icon: "🛡️",
                            title: "Reduces Piracy",
                            body: "When basics are free and genuinely useful, churches have no reason to seek workarounds."
                        }
                        StrategyCard {
                            icon: "📈",
                            title: "Grows Adoption",
                            body: "Free tier removes the barrier for small churches and youth ministries to get started."
                        }
                        StrategyCard {
                            icon: "💡",
                            title: "Teams Pay for Value",
                            body: "Organizations gladly pay for AI tools, collaboration, and broadcast features that save staffing hours."
                        }
                        StrategyCard {
                            icon: "🌱",
                            title: "Scales with You",
                            body: "The software becomes more powerful as your ministry grows — not more locked down."
                        }
                    }
                }

                // Premium categories
                div { class: "plans-categories",
                    h3 { class: "strategy-title", "What Premium Unlocks" }
                    div { class: "categories-grid",
                        CategoryCard { icon: "🤖", tier: "AI",         title: "AI Features",        items: &["Sermon generation", "AI study assistant", "Semantic search", "AI slide generation"] }
                        CategoryCard { icon: "📡", tier: "Standard",   title: "Broadcast & NDI",    items: &["Multiple outputs", "Alpha channel", "Livestream overlays", "Broadcast graphics"] }
                        CategoryCard { icon: "🎬", tier: "Standard",   title: "Advanced Media",     items: &["Cinematic templates", "Motion packs", "Animations", "Advanced lower thirds"] }
                        CategoryCard { icon: "👥", tier: "Enterprise", title: "Collaboration",      items: &["Team editing", "Cloud sync", "Approval workflows", "Shared libraries"] }
                        CategoryCard { icon: "🏢", tier: "Enterprise", title: "Organization",       items: &["Multi-campus", "Analytics", "Role management", "API access"] }
                        CategoryCard { icon: "☁️", tier: "AI",         title: "Cloud & AI Compute", items: &["Cloud backup", "Remote sync", "Template marketplace", "AI processing"] }
                    }
                }

                // Contact
                div { class: "plans-contact",
                    p { "Questions about which plan fits your ministry?" }
                    a {
                        class: "btn-primary",
                        href: "mailto:hoseahkplgt@gmail.com?subject=BiblePro%20Plan%20Inquiry",
                        "✉ Contact Us"
                    }
                }
            }
        }
    }
}

#[component]
fn StrategyCard(icon: &'static str, title: &'static str, body: &'static str) -> Element {
    rsx! {
        div { class: "strategy-card",
            span { class: "strategy-icon", "{icon}" }
            strong { class: "strategy-card-title", "{title}" }
            p { "{body}" }
        }
    }
}

#[component]
fn CategoryCard(icon: &'static str, tier: &'static str, title: &'static str, items: &'static [&'static str]) -> Element {
    rsx! {
        div { class: "category-card",
            div { class: "category-header",
                span { class: "category-icon", "{icon}" }
                span { class: "category-title", "{title}" }
                PremiumBadge { tier }
            }
            div { class: "category-items",
                for item in items {
                    div { class: "category-item", "· {item}" }
                }
            }
        }
    }
}
