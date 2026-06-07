use super::{db::BibleDb, models::*};
use tauri::State;

#[tauri::command]
pub fn get_translations(db: State<BibleDb>) -> Result<Vec<Translation>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name, abbreviation, language FROM translations ORDER BY name")
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], |row| {
            Ok(Translation {
                id: row.get(0)?,
                name: row.get(1)?,
                abbreviation: row.get(2)?,
                language: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(items)
}

#[tauri::command]
pub fn get_books(db: State<BibleDb>) -> Result<Vec<Book>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, short_name, testament, book_order FROM books ORDER BY book_order",
        )
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], |row| {
            Ok(Book {
                id: row.get(0)?,
                name: row.get(1)?,
                short_name: row.get(2)?,
                testament: row.get(3)?,
                book_order: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(items)
}

#[tauri::command]
pub fn get_chapter(
    db: State<BibleDb>,
    translation_id: String,
    book_id: i32,
    chapter: i32,
) -> Result<ChapterInfo, String> {
    let conn = db.0.lock().unwrap();

    let book_name: String = conn
        .query_row("SELECT name FROM books WHERE id = ?1", [book_id], |r| {
            r.get(0)
        })
        .map_err(|e| e.to_string())?;

    let total_chapters: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(chapter), 1) FROM verses WHERE translation_id = ?1 AND book_id = ?2",
            rusqlite::params![translation_id, book_id],
            |r| r.get(0),
        )
        .unwrap_or(1);

    let mut stmt = conn
        .prepare(
            "SELECT v.id, v.translation_id, v.book_id, b.name, v.chapter, v.verse, v.text
             FROM verses v JOIN books b ON v.book_id = b.id
             WHERE v.translation_id = ?1 AND v.book_id = ?2 AND v.chapter = ?3
             ORDER BY v.verse",
        )
        .map_err(|e| e.to_string())?;

    let verses = stmt
        .query_map(rusqlite::params![translation_id, book_id, chapter], |row| {
            Ok(Verse {
                id: row.get(0)?,
                translation_id: row.get(1)?,
                book_id: row.get(2)?,
                book_name: row.get(3)?,
                chapter: row.get(4)?,
                verse: row.get(5)?,
                text: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ChapterInfo {
        book_name,
        book_id,
        chapter,
        total_chapters,
        verses,
    })
}

#[tauri::command]
pub fn get_verse(
    db: State<BibleDb>,
    translation_id: String,
    book_id: i32,
    chapter: i32,
    verse: i32,
) -> Result<Option<Verse>, String> {
    let conn = db.0.lock().unwrap();
    let result = conn.query_row(
        "SELECT v.id, v.translation_id, v.book_id, b.name, v.chapter, v.verse, v.text
         FROM verses v JOIN books b ON v.book_id = b.id
         WHERE v.translation_id = ?1 AND v.book_id = ?2 AND v.chapter = ?3 AND v.verse = ?4",
        rusqlite::params![translation_id, book_id, chapter, verse],
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

#[tauri::command]
pub fn get_db_stats(db: State<BibleDb>) -> Result<DbStats, String> {
    let conn = db.0.lock().unwrap();

    let translation_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM translations", [], |r| r.get(0))
        .unwrap_or(0);
    let book_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0))
        .unwrap_or(0);
    let verse_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM verses", [], |r| r.get(0))
        .unwrap_or(0);

    let mut stmt = conn
        .prepare("SELECT id, name, abbreviation, language FROM translations ORDER BY name")
        .map_err(|e| e.to_string())?;
    let translations = stmt
        .query_map([], |row| {
            Ok(Translation {
                id: row.get(0)?,
                name: row.get(1)?,
                abbreviation: row.get(2)?,
                language: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(DbStats {
        translation_count,
        book_count,
        verse_count,
        translations,
    })
}
