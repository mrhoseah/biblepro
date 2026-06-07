use std::path::Path;

use serde::Serialize;

pub const CDN_BASE: &str = "https://cdn.jsdelivr.net/gh/thiagobodruk/bible@master/json";

#[derive(Debug, Clone, Serialize)]
pub struct CatalogEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub abbreviation: &'static str,
    pub language: &'static str,
    pub language_name: &'static str,
    pub category: &'static str,
    pub bundled: bool,
}

pub fn entries() -> &'static [CatalogEntry] {
    &[
        CatalogEntry {
            id: "kjv",
            name: "King James Version",
            abbreviation: "KJV",
            language: "en",
            language_name: "English",
            category: "Popular",
            bundled: true,
        },
        CatalogEntry {
            id: "asv",
            name: "American Standard Version",
            abbreviation: "ASV",
            language: "en",
            language_name: "English",
            category: "English",
            bundled: false,
        },
        CatalogEntry {
            id: "ylt",
            name: "Young's Literal Translation",
            abbreviation: "YLT",
            language: "en",
            language_name: "English",
            category: "English",
            bundled: false,
        },
        CatalogEntry {
            id: "bbe",
            name: "Bible in Basic English",
            abbreviation: "BBE",
            language: "en",
            language_name: "English",
            category: "English",
            bundled: false,
        },
        CatalogEntry {
            id: "darby",
            name: "Darby Bible",
            abbreviation: "DARBY",
            language: "en",
            language_name: "English",
            category: "English",
            bundled: false,
        },
    ]
}

pub fn source_url(entry: &CatalogEntry) -> String {
    format!("{CDN_BASE}/en_{}.json", entry.id)
}

pub fn by_id(id: &str) -> Option<&'static CatalogEntry> {
    let key = id.trim().to_lowercase();
    entries().iter().find(|e| e.id == key)
}

/// Guess translation metadata from a filename like `en_kjv.json` or `my-bible.sqlite`.
pub fn infer_import_metadata(path: &Path) -> (String, String, String) {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("import")
        .to_ascii_lowercase();

    for entry in entries() {
        if stem == entry.id
            || stem.ends_with(&format!("_{}", entry.id))
            || stem.contains(entry.id)
        {
            return (
                entry.id.to_string(),
                entry.name.to_string(),
                entry.language.to_string(),
            );
        }
    }

    if let Some(rest) = stem.strip_prefix("en_") {
        if let Some(entry) = by_id(rest) {
            return (
                entry.id.to_string(),
                entry.name.to_string(),
                entry.language.to_string(),
            );
        }
    }

    let id = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    let name = id
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let display_name = if name.is_empty() {
        "Imported Bible".to_string()
    } else {
        name
    };

    let safe_id = if id.is_empty() { "import".into() } else { id };
    (safe_id, display_name, "en".into())
}
