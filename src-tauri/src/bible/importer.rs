use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use super::db::BibleDb;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportResult {
    pub translation_id: String,
    pub verses_imported: usize,
    pub message: String,
}

/// thiagobodruk-style: array of books with `abbrev` + `chapters`.
#[derive(Deserialize)]
struct ThiagoBook {
    abbrev: Option<String>,
    chapters: Vec<Vec<String>>,
}

/// Simple positional format: array of books, each with `chapters`.
/// Book order = canonical order (Genesis first).
#[derive(Deserialize)]
struct SimpleBook {
    chapters: Vec<Vec<String>>,
}

// ── file-dialog import ────────────────────────────────────────────────────────

/// Opens a native file-picker, reads the chosen JSON file, and imports it.
/// Supports two formats:
///   - thiagobodruk  `[{"abbrev":"gn","chapters":[[...],...]}]`
///   - positional    `[{"chapters":[[...],...]}]`  (book order = canonical)
#[tauri::command]
pub async fn pick_and_import(
    app: tauri::AppHandle,
    db: State<'_, BibleDb>,
    translation_id: String,
    translation_name: String,
    language: String,
) -> Result<ImportResult, String> {
    // Open file picker (blocks until user picks or cancels)
    let path = app
        .dialog()
        .file()
        .add_filter("JSON Bible", &["json"])
        .blocking_pick_file()
        .ok_or("No file selected")?;

    let path_str = path.to_string();
    let data = std::fs::read(&path_str).map_err(|e| e.to_string())?;

    // Try thiagobodruk format first (has `abbrev`)
    if let Ok(books) = serde_json::from_slice::<Vec<ThiagoBook>>(&data) {
        if books.iter().any(|b| b.abbrev.is_some()) {
            // Reuse the seeder for thiagobodruk KJV-compatible layout
            // but with a custom translation_id — build a translation-aware seeder call.
            return import_thiagobodruk(&db, &data, &translation_id, &translation_name, &language);
        }
    }

    // Fall back to simple positional format
    import_positional(&db, &data, &translation_id, &translation_name, &language)
}

fn import_thiagobodruk(
    db: &State<BibleDb>,
    data: &[u8],
    translation_id: &str,
    translation_name: &str,
    language: &str,
) -> Result<ImportResult, String> {
    use super::seeder::abbrev_map_pub;
    let map = abbrev_map_pub();

    let bytes = if data.starts_with(b"\xef\xbb\xbf") { &data[3..] } else { data };
    let books: Vec<ThiagoBook> =
        serde_json::from_slice(bytes).map_err(|e| format!("JSON parse: {e}"))?;

    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO translations (id, name, abbreviation, language) VALUES (?1,?2,?3,?4)",
        rusqlite::params![translation_id, translation_name, translation_id.to_uppercase(), language],
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT OR REPLACE INTO verses (translation_id, book_id, chapter, verse, text)
             VALUES (?1,?2,?3,?4,?5)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for book in &books {
        let abbrev = book.abbrev.as_deref().unwrap_or("").to_ascii_lowercase();
        let book_id = match map.get(abbrev.as_str()) {
            Some(id) => *id,
            None => continue,
        };
        for (ch_idx, chapter) in book.chapters.iter().enumerate() {
            for (v_idx, text) in chapter.iter().enumerate() {
                stmt.execute(rusqlite::params![
                    translation_id, book_id,
                    (ch_idx + 1) as i32, (v_idx + 1) as i32, text
                ])
                .map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }

    Ok(ImportResult {
        translation_id: translation_id.to_string(),
        verses_imported: count,
        message: format!("Imported {} verses.", count),
    })
}

fn import_positional(
    db: &State<BibleDb>,
    data: &[u8],
    translation_id: &str,
    translation_name: &str,
    language: &str,
) -> Result<ImportResult, String> {
    let bytes = if data.starts_with(b"\xef\xbb\xbf") { &data[3..] } else { data };
    let books: Vec<SimpleBook> =
        serde_json::from_slice(bytes).map_err(|e| format!("JSON parse: {e}"))?;

    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO translations (id, name, abbreviation, language) VALUES (?1,?2,?3,?4)",
        rusqlite::params![translation_id, translation_name, translation_id.to_uppercase(), language],
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "INSERT OR REPLACE INTO verses (translation_id, book_id, chapter, verse, text)
             VALUES (?1,?2,?3,?4,?5)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for (book_idx, book) in books.iter().enumerate() {
        let book_id = (book_idx + 1) as i32;
        for (ch_idx, chapter) in book.chapters.iter().enumerate() {
            for (v_idx, text) in chapter.iter().enumerate() {
                stmt.execute(rusqlite::params![
                    translation_id, book_id,
                    (ch_idx + 1) as i32, (v_idx + 1) as i32, text
                ])
                .map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }

    Ok(ImportResult {
        translation_id: translation_id.to_string(),
        verses_imported: count,
        message: format!("Imported {} verses.", count),
    })
}

// ── API fetch ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn fetch_and_cache_passage(
    db: State<'_, BibleDb>,
    reference: String,
    translation_id: String,
) -> Result<String, String> {
    let encoded = reference.replace(' ', "+").replace(':', "%3A");
    let url = format!("https://bible-api.com/{encoded}");

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let verses_arr = json
        .get("verses")
        .and_then(|v| v.as_array())
        .ok_or("No verses in API response")?;

    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO translations (id, name, abbreviation, language) VALUES (?1,?2,?3,'en')",
        rusqlite::params![translation_id, "via bible-api.com", translation_id.to_uppercase()],
    )
    .map_err(|e| e.to_string())?;

    for v in verses_arr {
        let book_id = v.get("book_id").and_then(|b| b.as_i64()).unwrap_or(0) as i32;
        let chapter = v.get("chapter").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
        let verse   = v.get("verse").and_then(|n| n.as_i64()).unwrap_or(0) as i32;
        let text    = v.get("text").and_then(|t| t.as_str()).unwrap_or("");
        conn.execute(
            "INSERT OR REPLACE INTO verses (translation_id, book_id, chapter, verse, text)
             VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![translation_id, book_id, chapter, verse, text],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(json.get("text").and_then(|t| t.as_str()).unwrap_or("").trim().to_string())
}

// ── JSON-string import (frontend downloads the file, sends text here) ─────────
// This is the primary download path: the WASM frontend fetches the Bible JSON
// via the browser's fetch() API and passes the text to this command for parsing.

#[tauri::command]
pub fn import_from_json(
    db: State<BibleDb>,
    json_str: String,
    translation_id: String,
    translation_name: String,
    language: String,
) -> Result<ImportResult, String> {
    let data = json_str.as_bytes();

    // Try thiagobodruk format first (has "abbrev" field)
    if let Ok(books) = serde_json::from_slice::<Vec<ThiagoBook>>(data) {
        if books.iter().any(|b| b.abbrev.is_some()) {
            return import_thiagobodruk(&db, data, &translation_id, &translation_name, &language);
        }
    }

    // Fall back to positional format
    import_positional(&db, data, &translation_id, &translation_name, &language)
}
