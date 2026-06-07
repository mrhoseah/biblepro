use super::{db::BibleDb, models::*};
use tauri::State;

fn fts_search(
    conn: &rusqlite::Connection,
    fts_q: &str,
    translation_id: &str,
    book_id: Option<i32>,
    testament: Option<&str>,
    limit: i32,
) -> Result<Vec<SearchResult>, String> {
    // (?N IS NULL OR col = ?N) lets optional filters be passed as NULL
    let sql = "SELECT v.id, v.translation_id, v.book_id, b.name, v.chapter, v.verse, v.text,
                highlight(verses_fts, 0, '\u{ab}', '\u{bb}')
         FROM verses_fts
         JOIN verses v ON verses_fts.rowid = v.id
         JOIN books b ON v.book_id = b.id
         WHERE verses_fts MATCH ?1
           AND v.translation_id = ?2
           AND (?3 IS NULL OR v.book_id = ?3)
           AND (?4 IS NULL OR b.testament = ?4)
         ORDER BY rank
         LIMIT ?5";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    stmt.query_map(
        rusqlite::params![fts_q, translation_id, book_id, testament, limit],
        |row| {
            let text: String = row.get(6)?;
            let snippet: String = row.get(7)?;
            Ok(SearchResult {
                verse: Verse {
                    id: row.get(0)?,
                    translation_id: row.get(1)?,
                    book_id: row.get(2)?,
                    book_name: row.get(3)?,
                    chapter: row.get(4)?,
                    verse: row.get(5)?,
                    text,
                },
                snippet,
            })
        },
    )
    .map_err(|e| e.to_string())
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn search_verses(
    db: State<BibleDb>,
    translation_id: String,
    query: String,
    limit: Option<i32>,
    book_id: Option<i32>,
    testament: Option<String>,
) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let conn = db.0.lock().unwrap();
    let max = limit.unwrap_or(50);
    let t = testament.as_deref();

    // Phrase search first; fall back to bare terms
    let phrase = format!("\"{}\"", query.replace('"', ""));
    match fts_search(&conn, &phrase, &translation_id, book_id, t, max) {
        Ok(r) if !r.is_empty() => Ok(r),
        _ => fts_search(&conn, query.trim(), &translation_id, book_id, t, max),
    }
}

#[tauri::command]
pub fn search_by_reference(
    db: State<BibleDb>,
    translation_id: String,
    reference: String,
) -> Result<Option<Verse>, String> {
    let conn = db.0.lock().unwrap();

    // Parse "Book Chapter:Verse" — split on last ':'
    let parts: Vec<&str> = reference.trim().rsplitn(2, ':').collect();
    if parts.len() != 2 {
        return Ok(None);
    }

    let verse_num: i32 = parts[0].trim().parse().unwrap_or(0);
    let before_colon = parts[1].trim();

    // Split on last space to separate book from chapter
    let ch_split: Vec<&str> = before_colon.rsplitn(2, ' ').collect();
    if ch_split.len() != 2 {
        return Ok(None);
    }

    let chapter_num: i32 = ch_split[0].trim().parse().unwrap_or(0);
    let book_query = format!("%{}%", ch_split[1].trim());

    let result = conn.query_row(
        "SELECT v.id, v.translation_id, v.book_id, b.name, v.chapter, v.verse, v.text
         FROM verses v JOIN books b ON v.book_id = b.id
         WHERE v.translation_id = ?1
           AND b.name LIKE ?2
           AND v.chapter = ?3
           AND v.verse = ?4
         LIMIT 1",
        rusqlite::params![translation_id, book_query, chapter_num, verse_num],
        |row| {
            Ok(Verse {
                id: row.get(0)?,
                translation_id: row.get(1)?,
                book_id: row.get(2)?,
                book_name: row.get(3)?,
                chapter: row.get(4)?,
                verse: row.get(5)?,
                text: row.get(6)?,
            })
        },
    );

    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
