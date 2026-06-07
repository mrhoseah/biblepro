use super::catalog::{self, CatalogEntry};
use super::db::BibleDb;
use super::seeder;
use rusqlite::{types::ValueRef, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportResult {
    pub translation_id: String,
    pub verses_imported: usize,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct InstallBibleRequest {
    pub translation_id: String,
    pub translation_name: String,
    pub abbreviation: String,
    pub language: String,
    pub source_url: String,
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

// ── catalog + simplified install/import ───────────────────────────────────────

#[tauri::command]
pub fn list_bible_catalog() -> Vec<CatalogEntry> {
    catalog::entries().to_vec()
}

/// One-click install from the built-in catalog (downloads over HTTPS or uses bundled data).
#[tauri::command]
pub async fn install_bible(
    db: State<'_, BibleDb>,
    translation_id: String,
) -> Result<ImportResult, String> {
    let key = translation_id.trim().to_lowercase();
    let entry = catalog::by_id(&key).ok_or_else(|| format!("Unknown Bible: {key}"))?;

    if entry.bundled {
        let conn = db.0.lock().unwrap();
        let verses = seeder::seed_translation(
            &conn,
            entry.id,
            entry.name,
            entry.abbreviation,
            entry.language,
            include_bytes!("../../resources/en_kjv.json"),
        )?;
        if verses > 0 {
            return Ok(ImportResult {
                translation_id: entry.id.to_string(),
                verses_imported: verses,
                message: format!(
                    "Installed {} with {} verses. Ready offline.",
                    entry.abbreviation, verses
                ),
            });
        }
        drop(conn);
        let existing: i64 = db
            .0
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM verses WHERE translation_id = ?1",
                [entry.id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if existing > 0 {
            return Ok(ImportResult {
                translation_id: entry.id.to_string(),
                verses_imported: existing as usize,
                message: format!("{} is already installed.", entry.name),
            });
        }
    }

    install_bible_from_url(
        db,
        InstallBibleRequest {
            translation_id: entry.id.to_string(),
            translation_name: entry.name.to_string(),
            abbreviation: entry.abbreviation.to_string(),
            language: entry.language.to_string(),
            source_url: catalog::source_url(entry),
        },
    )
    .await
}

/// Pick a Bible file and import it. Translation metadata is inferred from the filename when possible.
#[tauri::command]
pub async fn import_bible_file(
    app: tauri::AppHandle,
    db: State<'_, BibleDb>,
) -> Result<ImportResult, String> {
    let path = app
        .dialog()
        .file()
        .add_filter(
            "Bible files",
            &[
                "json", "sqlite", "sqlite3", "db", "bbl", "bblx", "bible", "txt", "tsv", "csv",
            ],
        )
        .blocking_pick_file()
        .ok_or("No file selected")?;

    let path_str = path.to_string();
    let data = std::fs::read(&path_str).map_err(|e| e.to_string())?;
    let (translation_id, translation_name, language) =
        catalog::infer_import_metadata(Path::new(&path_str));
    import_from_path_or_bytes(
        &db,
        Path::new(&path_str),
        &data,
        &translation_id,
        &translation_name,
        &language,
    )
}

// ── file-dialog import ────────────────────────────────────────────────────────

/// Opens a native file-picker, reads the chosen Bible file, and imports it.
/// Supports:
///   - thiagobodruk  `[{"abbrev":"gn","chapters":[[...],...]}]`
///   - positional    `[{"chapters":[[...],...]}]`  (book order = canonical)
///   - SQLite Bible modules with book/chapter/verse/text columns
///   - tab/CSV/text rows: book, chapter, verse, text
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
        .add_filter(
            "Bible modules",
            &[
                "json", "sqlite", "sqlite3", "db", "bbl", "bblx", "bible", "txt", "tsv", "csv",
            ],
        )
        .blocking_pick_file()
        .ok_or("No file selected")?;

    let path_str = path.to_string();
    let data = std::fs::read(&path_str).map_err(|e| e.to_string())?;
    import_from_path_or_bytes(
        &db,
        Path::new(&path_str),
        &data,
        &translation_id,
        &translation_name,
        &language,
    )
}

fn import_from_path_or_bytes(
    db: &State<BibleDb>,
    path: &Path,
    data: &[u8],
    translation_id: &str,
    translation_name: &str,
    language: &str,
) -> Result<ImportResult, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if matches!(
        extension.as_str(),
        "sqlite" | "sqlite3" | "db" | "bbl" | "bblx" | "bible"
    ) {
        if let Ok(result) =
            import_sqlite_module(db, path, translation_id, translation_name, language)
        {
            return Ok(result);
        }
    }

    import_from_bytes(db, data, translation_id, translation_name, language)
}

fn import_from_bytes(
    db: &State<BibleDb>,
    data: &[u8],
    translation_id: &str,
    translation_name: &str,
    language: &str,
) -> Result<ImportResult, String> {
    if let Ok(books) = serde_json::from_slice::<Vec<ThiagoBook>>(data) {
        if books.iter().any(|b| b.abbrev.is_some()) {
            return import_thiagobodruk(&db, &data, &translation_id, &translation_name, &language);
        }
    }

    if serde_json::from_slice::<Vec<SimpleBook>>(data).is_ok() {
        return import_positional(&db, &data, &translation_id, &translation_name, &language);
    }

    import_delimited_text(db, data, translation_id, translation_name, language)
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

    let bytes = if data.starts_with(b"\xef\xbb\xbf") {
        &data[3..]
    } else {
        data
    };
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
                    translation_id,
                    book_id,
                    (ch_idx + 1) as i32,
                    (v_idx + 1) as i32,
                    text
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
    let bytes = if data.starts_with(b"\xef\xbb\xbf") {
        &data[3..]
    } else {
        data
    };
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
                    translation_id,
                    book_id,
                    (ch_idx + 1) as i32,
                    (v_idx + 1) as i32,
                    text
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

#[derive(Debug)]
struct FlatVerse {
    book_id: i32,
    chapter: i32,
    verse: i32,
    text: String,
}

fn import_sqlite_module(
    db: &State<BibleDb>,
    path: &Path,
    translation_id: &str,
    translation_name: &str,
    language: &str,
) -> Result<ImportResult, String> {
    let source = Connection::open(path).map_err(|e| format!("Open Bible module database: {e}"))?;
    let tables = sqlite_tables(&source)?;

    for table in tables {
        let columns = table_columns(&source, &table)?;
        if let Some(mapping) = detect_verse_columns(&columns) {
            let verses = read_sqlite_verses(&source, &table, &mapping)?;
            if !verses.is_empty() {
                return import_flat_verses(db, verses, translation_id, translation_name, language);
            }
        }
    }

    Err(
        "No supported Bible verse table found. Expected columns like book, chapter, verse, text."
            .to_string(),
    )
}

fn sqlite_tables(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>, String> {
    let sql = format!("PRAGMA table_info({})", quote_ident(table));
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[derive(Debug)]
struct VerseColumnMapping {
    book: String,
    chapter: String,
    verse: String,
    text: String,
}

fn detect_verse_columns(columns: &[String]) -> Option<VerseColumnMapping> {
    Some(VerseColumnMapping {
        book: find_column(
            columns,
            &[
                "book_id",
                "bookid",
                "book_number",
                "booknumber",
                "book",
                "b",
                "book_nr",
                "booknr",
            ],
        )?,
        chapter: find_column(
            columns,
            &[
                "chapter",
                "chapter_number",
                "chapternumber",
                "chapter_id",
                "c",
                "chap",
            ],
        )?,
        verse: find_column(
            columns,
            &[
                "verse",
                "verse_number",
                "versenumber",
                "verse_id",
                "v",
                "vers",
            ],
        )?,
        text: find_column(
            columns,
            &[
                "text",
                "verse_text",
                "versetext",
                "scripture",
                "content",
                "body",
            ],
        )?,
    })
}

fn find_column(columns: &[String], candidates: &[&str]) -> Option<String> {
    candidates.iter().find_map(|candidate| {
        columns
            .iter()
            .find(|column| normalize_key(column) == normalize_key(candidate))
            .cloned()
    })
}

fn read_sqlite_verses(
    conn: &Connection,
    table: &str,
    mapping: &VerseColumnMapping,
) -> Result<Vec<FlatVerse>, String> {
    let sql = format!(
        "SELECT {}, {}, {}, {} FROM {}",
        quote_ident(&mapping.book),
        quote_ident(&mapping.chapter),
        quote_ident(&mapping.verse),
        quote_ident(&mapping.text),
        quote_ident(table),
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let book_id = value_ref_to_book_id(row.get_ref(0)?)?;
            let chapter = value_ref_to_i32(row.get_ref(1)?)?;
            let verse = value_ref_to_i32(row.get_ref(2)?)?;
            let text = value_ref_to_string(row.get_ref(3)?)?;
            Ok(FlatVerse {
                book_id,
                chapter,
                verse,
                text,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut verses = Vec::new();
    for row in rows {
        let verse = row.map_err(|e| e.to_string())?;
        if verse.book_id > 0
            && verse.chapter > 0
            && verse.verse > 0
            && !verse.text.trim().is_empty()
        {
            verses.push(verse);
        }
    }
    Ok(verses)
}

fn import_delimited_text(
    db: &State<BibleDb>,
    data: &[u8],
    translation_id: &str,
    translation_name: &str,
    language: &str,
) -> Result<ImportResult, String> {
    let text = std::str::from_utf8(strip_bom(data)).map_err(|e| format!("Text parse: {e}"))?;
    let mut verses = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(verse) = parse_delimited_line(line) {
            verses.push(verse);
        }
    }

    if verses.is_empty() {
        return Err("Unsupported import format. Use JSON, SQLite BibleShow-style modules, or rows with book/chapter/verse/text.".to_string());
    }

    import_flat_verses(db, verses, translation_id, translation_name, language)
}

fn parse_delimited_line(line: &str) -> Option<FlatVerse> {
    for delimiter in ['\t', '|', ','] {
        let parts: Vec<&str> = line.split(delimiter).map(str::trim).collect();
        if parts.len() >= 4 {
            let book_id = parse_book_id(parts[0])?;
            let chapter = parts[1].parse::<i32>().ok()?;
            let verse = parts[2].parse::<i32>().ok()?;
            let text = parts[3..].join(" ");
            return Some(FlatVerse {
                book_id,
                chapter,
                verse,
                text,
            });
        }
    }

    parse_reference_line(line)
}

fn parse_reference_line(line: &str) -> Option<FlatVerse> {
    let colon = line.find(':')?;
    let before_colon = &line[..colon];
    let after_colon = &line[colon + 1..];
    let mut left_parts: Vec<&str> = before_colon.split_whitespace().collect();
    let chapter = left_parts.pop()?.parse::<i32>().ok()?;
    let book = left_parts.join(" ");
    let mut after_parts = after_colon.splitn(2, char::is_whitespace);
    let verse = after_parts.next()?.trim().parse::<i32>().ok()?;
    let text = after_parts.next()?.trim().to_string();
    Some(FlatVerse {
        book_id: parse_book_id(&book)?,
        chapter,
        verse,
        text,
    })
}

fn import_flat_verses(
    db: &State<BibleDb>,
    verses: Vec<FlatVerse>,
    translation_id: &str,
    translation_name: &str,
    language: &str,
) -> Result<ImportResult, String> {
    let translation_id = translation_id.trim().to_lowercase();
    let conn = db.0.lock().unwrap();
    conn.execute(
        "DELETE FROM verses WHERE translation_id = ?1",
        rusqlite::params![translation_id],
    )
    .map_err(|e| e.to_string())?;
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
    for verse in verses {
        stmt.execute(rusqlite::params![
            translation_id,
            verse.book_id,
            verse.chapter,
            verse.verse,
            verse.text.trim()
        ])
        .map_err(|e| e.to_string())?;
        count += 1;
    }

    Ok(ImportResult {
        translation_id,
        verses_imported: count,
        message: format!("Imported {} verses.", count),
    })
}

fn strip_bom(data: &[u8]) -> &[u8] {
    if data.starts_with(b"\xef\xbb\xbf") {
        &data[3..]
    } else {
        data
    }
}

fn quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn normalize_key(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn value_ref_to_i32(value: ValueRef<'_>) -> rusqlite::Result<i32> {
    match value {
        ValueRef::Integer(value) => Ok(value as i32),
        ValueRef::Real(value) => Ok(value as i32),
        ValueRef::Text(value) => Ok(std::str::from_utf8(value)
            .ok()
            .and_then(|value| value.trim().parse::<i32>().ok())
            .unwrap_or(0)),
        _ => Ok(0),
    }
}

fn value_ref_to_string(value: ValueRef<'_>) -> rusqlite::Result<String> {
    match value {
        ValueRef::Text(value) => Ok(String::from_utf8_lossy(value).to_string()),
        ValueRef::Integer(value) => Ok(value.to_string()),
        ValueRef::Real(value) => Ok(value.to_string()),
        ValueRef::Blob(value) => Ok(String::from_utf8_lossy(value).to_string()),
        ValueRef::Null => Ok(String::new()),
    }
}

fn value_ref_to_book_id(value: ValueRef<'_>) -> rusqlite::Result<i32> {
    match value {
        ValueRef::Integer(value) => Ok(value as i32),
        ValueRef::Real(value) => Ok(value as i32),
        ValueRef::Text(value) => {
            let text = String::from_utf8_lossy(value);
            Ok(parse_book_id(&text).unwrap_or(0))
        }
        _ => Ok(0),
    }
}

fn parse_book_id(value: &str) -> Option<i32> {
    let trimmed = value.trim();
    if let Ok(book_id) = trimmed.parse::<i32>() {
        if (1..=66).contains(&book_id) {
            return Some(book_id);
        }
    }

    let key = normalize_key(trimmed);
    let books = [
        (1, &["genesis", "gen", "ge", "gn"][..]),
        (2, &["exodus", "exo", "ex"]),
        (3, &["leviticus", "lev", "le", "lv"]),
        (4, &["numbers", "num", "nu", "nm", "nb"]),
        (5, &["deuteronomy", "deut", "deu", "dt"]),
        (6, &["joshua", "josh", "jos"]),
        (7, &["judges", "judg", "jdg"]),
        (8, &["ruth", "rut"]),
        (9, &["1samuel", "1sam", "1sa", "isamuel", "isam"]),
        (10, &["2samuel", "2sam", "2sa", "iisamuel", "iisam"]),
        (11, &["1kings", "1kgs", "1ki", "ikings"]),
        (12, &["2kings", "2kgs", "2ki", "iikings"]),
        (13, &["1chronicles", "1chron", "1ch", "ichronicles"]),
        (14, &["2chronicles", "2chron", "2ch", "iichronicles"]),
        (15, &["ezra", "ezr"]),
        (16, &["nehemiah", "neh"]),
        (17, &["esther", "est"]),
        (18, &["job"]),
        (19, &["psalms", "psalm", "ps", "psa", "psm"]),
        (20, &["proverbs", "prov", "pro", "prv"]),
        (21, &["ecclesiastes", "eccl", "ecc"]),
        (22, &["songofsolomon", "songofsongs", "song", "sos", "sol"]),
        (23, &["isaiah", "isa", "is"]),
        (24, &["jeremiah", "jer", "je", "jr"]),
        (25, &["lamentations", "lam", "la"]),
        (26, &["ezekiel", "ezek", "eze", "ezk"]),
        (27, &["daniel", "dan", "da", "dn"]),
        (28, &["hosea", "hos"]),
        (29, &["joel", "joe"]),
        (30, &["amos", "amo"]),
        (31, &["obadiah", "obad", "oba"]),
        (32, &["jonah", "jon"]),
        (33, &["micah", "mic"]),
        (34, &["nahum", "nah"]),
        (35, &["habakkuk", "hab"]),
        (36, &["zephaniah", "zep", "zeph"]),
        (37, &["haggai", "hag"]),
        (38, &["zechariah", "zec", "zech"]),
        (39, &["malachi", "mal"]),
        (40, &["matthew", "matt", "mat", "mt"]),
        (41, &["mark", "mar", "mk"]),
        (42, &["luke", "luk", "lk"]),
        (43, &["john", "joh", "jn"]),
        (44, &["acts", "act"]),
        (45, &["romans", "rom", "ro", "rm"]),
        (46, &["1corinthians", "1cor", "1co", "icorinthians"]),
        (47, &["2corinthians", "2cor", "2co", "iicorinthians"]),
        (48, &["galatians", "gal"]),
        (49, &["ephesians", "eph"]),
        (50, &["philippians", "phil", "phi", "php"]),
        (51, &["colossians", "col"]),
        (52, &["1thessalonians", "1thess", "1th", "ithessalonians"]),
        (53, &["2thessalonians", "2thess", "2th", "iithessalonians"]),
        (54, &["1timothy", "1tim", "1ti", "itimothy"]),
        (55, &["2timothy", "2tim", "2ti", "iitimothy"]),
        (56, &["titus", "tit"]),
        (57, &["philemon", "phlm", "phm"]),
        (58, &["hebrews", "heb"]),
        (59, &["james", "jas", "jam"]),
        (60, &["1peter", "1pet", "1pe", "ipeter"]),
        (61, &["2peter", "2pet", "2pe", "iipeter"]),
        (62, &["1john", "1jn", "1jo", "ijohn"]),
        (63, &["2john", "2jn", "2jo", "iijohn"]),
        (64, &["3john", "3jn", "3jo", "iiijohn"]),
        (65, &["jude", "jud"]),
        (66, &["revelation", "rev", "re"]),
    ];

    books.iter().find_map(|(id, aliases)| {
        aliases
            .iter()
            .any(|alias| normalize_key(alias) == key)
            .then_some(*id)
    })
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

#[tauri::command]
pub async fn install_bible_from_url(
    db: State<'_, BibleDb>,
    request: InstallBibleRequest,
) -> Result<ImportResult, String> {
    let translation_id = request.translation_id.trim().to_lowercase();
    let source_url = request.source_url.trim();

    if translation_id.is_empty()
        || request.translation_name.trim().is_empty()
        || request.abbreviation.trim().is_empty()
        || source_url.is_empty()
    {
        return Err(
            "translation_id, translation_name, abbreviation, and source_url are required"
                .to_string(),
        );
    }

    if !source_url.starts_with("https://") {
        return Err("Bible datasets must be downloaded over HTTPS".to_string());
    }

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(source_url)
        .header("User-Agent", "BiblePro/0.1 Bible Package Installer")
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status {}", response.status()));
    }

    let data = response
        .bytes()
        .await
        .map_err(|e| format!("Read download: {e}"))?;

    let is_thiago_format = if let Ok(books) = serde_json::from_slice::<Vec<ThiagoBook>>(&data) {
        books.iter().any(|book| book.abbrev.is_some())
    } else {
        serde_json::from_slice::<Vec<SimpleBook>>(&data)
            .map_err(|e| format!("Downloaded Bible JSON was not a supported format: {e}"))?;
        false
    };

    remove_translation_data(&db, &translation_id)?;

    let result = if is_thiago_format {
        import_thiagobodruk(
            &db,
            &data,
            &translation_id,
            request.translation_name.trim(),
            request.language.trim(),
        )?
    } else {
        import_positional(
            &db,
            &data,
            &translation_id,
            request.translation_name.trim(),
            request.language.trim(),
        )?
    };

    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE translations SET abbreviation = ?1 WHERE id = ?2",
        rusqlite::params![request.abbreviation.trim().to_uppercase(), translation_id],
    )
    .map_err(|e| e.to_string())?;
    drop(conn);

    if result.verses_imported == 0 {
        remove_translation_data(&db, &result.translation_id)?;
        return Err("Downloaded dataset did not import any verses".to_string());
    }

    Ok(ImportResult {
        message: format!(
            "Installed {} with {} verses. Ready offline.",
            request.abbreviation.trim().to_uppercase(),
            result.verses_imported
        ),
        ..result
    })
}

#[tauri::command]
pub fn remove_translation(
    db: State<BibleDb>,
    translation_id: String,
) -> Result<ImportResult, String> {
    let translation_id = translation_id.trim().to_lowercase();
    if translation_id.is_empty() {
        return Err("translation_id is required".to_string());
    }

    let removed = remove_translation_data(&db, &translation_id)?;
    Ok(ImportResult {
        translation_id,
        verses_imported: removed,
        message: format!("Removed translation and {} verses.", removed),
    })
}

fn remove_translation_data(db: &State<BibleDb>, translation_id: &str) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    let removed = conn
        .execute(
            "DELETE FROM verses WHERE translation_id = ?1",
            rusqlite::params![translation_id],
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM translations WHERE id = ?1",
        rusqlite::params![translation_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(removed)
}
