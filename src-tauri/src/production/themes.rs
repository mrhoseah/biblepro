use crate::present::config::{BackgroundDesign, BgMode, Rgba};

use super::motion::motion_id_for_media;
use super::models::{CountdownDef, CountdownStyle, MediaDef, ProductionTheme};

fn rgba(r: u8, g: u8, b: u8) -> Rgba {
    Rgba::new(r, g, b, 255)
}

pub fn builtin_themes() -> Vec<ProductionTheme> {
    vec![
        ProductionTheme {
            id: "worship-glow".into(),
            name: "Worship Glow".into(),
            background: BackgroundDesign::two_stop(
                BgMode::Diagonal,
                rgba(59, 130, 246),
                rgba(30, 27, 75),
            ),
            headline_color: rgba(255, 255, 255),
            timer_color: rgba(255, 255, 255),
            subline_color: rgba(200, 210, 255),
        },
        ProductionTheme {
            id: "youth-energy".into(),
            name: "Youth Energy".into(),
            background: BackgroundDesign::two_stop(
                BgMode::Diagonal,
                rgba(217, 70, 239),
                rgba(88, 28, 135),
            ),
            headline_color: rgba(255, 255, 255),
            timer_color: rgba(255, 255, 255),
            subline_color: rgba(250, 200, 255),
        },
        ProductionTheme {
            id: "conference-minimal".into(),
            name: "Conference Minimal".into(),
            background: BackgroundDesign::two_stop(
                BgMode::LinearV,
                rgba(6, 182, 212),
                rgba(15, 23, 42),
            ),
            headline_color: rgba(220, 240, 255),
            timer_color: rgba(255, 255, 255),
            subline_color: rgba(148, 163, 184),
        },
        ProductionTheme {
            id: "prayer-soft".into(),
            name: "Prayer Soft Fade".into(),
            background: BackgroundDesign::two_stop(
                BgMode::Radial,
                rgba(168, 85, 247),
                rgba(59, 7, 100),
            ),
            headline_color: rgba(240, 230, 255),
            timer_color: rgba(255, 255, 255),
            subline_color: rgba(200, 180, 230),
        },
        ProductionTheme {
            id: "blackout".into(),
            name: "Blackout Mode".into(),
            background: BackgroundDesign::solid(rgba(0, 0, 0)),
            headline_color: rgba(180, 180, 180),
            timer_color: rgba(255, 255, 255),
            subline_color: rgba(120, 120, 120),
        },
        ProductionTheme {
            id: "broadcast".into(),
            name: "Broadcast".into(),
            background: BackgroundDesign::two_stop(
                BgMode::LinearH,
                rgba(239, 68, 68),
                rgba(0, 0, 0),
            ),
            headline_color: rgba(255, 220, 220),
            timer_color: rgba(255, 255, 255),
            subline_color: rgba(200, 200, 200),
        },
        ProductionTheme {
            id: "classic-church".into(),
            name: "Classic Church".into(),
            background: BackgroundDesign::two_stop(
                BgMode::Radial,
                rgba(252, 211, 77),
                rgba(124, 45, 18),
            ),
            headline_color: rgba(255, 250, 230),
            timer_color: rgba(255, 255, 255),
            subline_color: rgba(255, 230, 180),
        },
    ]
}

pub fn theme_by_id(id: &str) -> ProductionTheme {
    builtin_themes()
        .into_iter()
        .find(|t| t.id == id)
        .unwrap_or_else(|| builtin_themes()[0].clone())
}

pub fn builtin_media() -> Vec<MediaDef> {
    vec![
        MediaDef {
            id: "blue-motion".into(),
            title: "Blue Worship Motion".into(),
            category: "Worship".into(),
            media_type: "video".into(),
            background: BackgroundDesign::two_stop(
                BgMode::Diagonal,
                rgba(59, 130, 246),
                rgba(30, 58, 138),
            ),
            motion_id: motion_id_for_media("blue-motion").map(str::to_string),
        },
        MediaDef {
            id: "gold-clouds".into(),
            title: "Golden Clouds".into(),
            category: "Scripture".into(),
            media_type: "image".into(),
            background: BackgroundDesign::two_stop(
                BgMode::Radial,
                rgba(252, 211, 77),
                rgba(154, 52, 18),
            ),
            motion_id: None,
        },
        MediaDef {
            id: "purple-prayer".into(),
            title: "Purple Prayer Gradient".into(),
            category: "Prayer".into(),
            media_type: "gradient".into(),
            background: BackgroundDesign::two_stop(
                BgMode::LinearV,
                rgba(168, 85, 247),
                rgba(112, 26, 117),
            ),
            motion_id: None,
        },
        MediaDef {
            id: "solid-black".into(),
            title: "Solid Black".into(),
            category: "Scripture".into(),
            media_type: "color".into(),
            background: BackgroundDesign::solid(rgba(0, 0, 0)),
            motion_id: None,
        },
        MediaDef {
            id: "conference-lines".into(),
            title: "Conference Lines".into(),
            category: "Conference".into(),
            media_type: "video".into(),
            background: BackgroundDesign::two_stop(
                BgMode::Diagonal,
                rgba(6, 182, 212),
                rgba(30, 41, 59),
            ),
            motion_id: motion_id_for_media("conference-lines").map(str::to_string),
        },
        MediaDef {
            id: "countdown-rays".into(),
            title: "Countdown Light Rays".into(),
            category: "Countdowns".into(),
            media_type: "video".into(),
            background: BackgroundDesign::two_stop(
                BgMode::Radial,
                rgba(56, 189, 248),
                rgba(0, 0, 0),
            ),
            motion_id: motion_id_for_media("countdown-rays").map(str::to_string),
        },
    ]
}

pub fn media_by_id(id: &str) -> Option<MediaDef> {
    builtin_media().into_iter().find(|m| m.id == id)
}

pub fn builtin_countdowns() -> Vec<CountdownDef> {
    vec![
        CountdownDef {
            id: "sunday-service".into(),
            name: "Sunday Service".into(),
            duration: 600,
            style: CountdownStyle::Ring,
            theme_id: "worship-glow".into(),
            headline: "Service Starts In".into(),
            subline: "Welcome to Sunday Worship".into(),
            loader: "Ring".into(),
            media_id: Some("blue-motion".into()),
        },
        CountdownDef {
            id: "youth-night".into(),
            name: "Youth Night".into(),
            duration: 300,
            style: CountdownStyle::Loader,
            theme_id: "youth-energy".into(),
            headline: "YOUTH NIGHT".into(),
            subline: "Get ready".into(),
            loader: "Wave".into(),
            media_id: Some("conference-lines".into()),
        },
        CountdownDef {
            id: "conference-session".into(),
            name: "Conference Session".into(),
            duration: 900,
            style: CountdownStyle::Numeric,
            theme_id: "conference-minimal".into(),
            headline: "Session Begins In".into(),
            subline: "Main auditorium".into(),
            loader: "Minimal Line".into(),
            media_id: Some("conference-lines".into()),
        },
        CountdownDef {
            id: "livestream".into(),
            name: "Livestream".into(),
            duration: 120,
            style: CountdownStyle::Numeric,
            theme_id: "broadcast".into(),
            headline: "We're Going Live".into(),
            subline: "Share this service with a friend".into(),
            loader: "Progress Bar".into(),
            media_id: Some("solid-black".into()),
        },
        CountdownDef {
            id: "sermon".into(),
            name: "Message Countdown".into(),
            duration: 180,
            style: CountdownStyle::Loader,
            theme_id: "classic-church".into(),
            headline: "Message Begins In".into(),
            subline: "Prepare your Bible".into(),
            loader: "Pulse".into(),
            media_id: Some("gold-clouds".into()),
        },
    ]
}

pub fn countdown_by_id(id: &str) -> Option<CountdownDef> {
    builtin_countdowns().into_iter().find(|c| c.id == id)
}
