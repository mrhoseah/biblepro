use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlanItemKind {
    Verse {
        translation_id: String,
        book_id: i32,
        chapter: i32,
        verse: i32,
        reference: String,
        text: String,
    },
    Song {
        song_id: u64,
        title: String,
        section_label: Option<String>,
    },
    Countdown {
        countdown_id: String,
        name: String,
    },
    Media {
        media_id: String,
        title: String,
    },
    Blank {
        label: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePlanItem {
    pub id: String,
    pub kind: PlanItemKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePlan {
    pub name: String,
    pub items: Vec<ServicePlanItem>,
}

impl Default for ServicePlan {
    fn default() -> Self {
        Self {
            name: "Sunday Service".into(),
            items: Vec::new(),
        }
    }
}
