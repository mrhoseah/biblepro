use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct ThiagoBook {
    abbrev: String,
    chapters: Vec<Vec<String>>,
}

/// Maps thiagobodruk abbreviations → canonical book IDs (1-indexed).
pub fn abbrev_map_pub() -> HashMap<&'static str, i32> {
    abbrev_map()
}

fn abbrev_map() -> HashMap<&'static str, i32> {
    let pairs = [
        ("gn", 1),
        ("ex", 2),
        ("lv", 3),
        ("nm", 4),
        ("dt", 5),
        ("js", 6),
        ("jud", 7),
        ("rt", 8),
        ("1sm", 9),
        ("2sm", 10),
        ("1kgs", 11),
        ("2kgs", 12),
        ("1ch", 13),
        ("2ch", 14),
        ("ez", 15),
        ("ne", 16),
        ("et", 17),
        ("job", 18),
        ("ps", 19),
        ("prv", 20),
        ("ec", 21),
        ("sg", 22),
        ("is", 23),
        ("jr", 24),
        ("lm", 25),
        ("ezk", 26),
        ("dn", 27),
        ("ho", 28),
        ("jl", 29),
        ("am", 30),
        ("ob", 31),
        ("jn", 32),
        ("mi", 33),
        ("na", 34),
        ("hk", 35),
        ("zp", 36),
        ("hg", 37),
        ("zc", 38),
        ("ml", 39),
        ("mt", 40),
        ("mk", 41),
        ("lk", 42),
        ("jo", 43),
        ("act", 44),
        ("rm", 45),
        ("1co", 46),
        ("2co", 47),
        ("gl", 48),
        ("ep", 49),
        ("ph", 50),
        ("cl", 51),
        ("1ts", 52),
        ("2ts", 53),
        ("1tm", 54),
        ("2tm", 55),
        ("tt", 56),
        ("phm", 57),
        ("hb", 58),
        ("jm", 59),
        ("1pe", 60),
        ("2pe", 61),
        ("1jo", 62),
        ("2jo", 63),
        ("3jo", 64),
        ("jd", 65),
        ("rv", 66),
    ];
    pairs.iter().cloned().collect()
}

/// Seed bundled translations when the database has no Bible text yet.
pub fn seed_bundled(conn: &Connection) -> Result<usize, String> {
    let translations: i64 = conn
        .query_row("SELECT COUNT(*) FROM translations", [], |r| r.get(0))
        .unwrap_or(0);
    if translations > 0 {
        return Ok(0);
    }
    seed_translation(
        conn,
        "kjv",
        "King James Version",
        "KJV",
        "en",
        include_bytes!("../../resources/en_kjv.json"),
    )
}

/// Insert a thiagobodruk-format Bible from JSON bytes.
pub fn seed_translation(
    conn: &Connection,
    translation_id: &str,
    translation_name: &str,
    abbreviation: &str,
    language: &str,
    json_bytes: &[u8],
) -> Result<usize, String> {
    let already: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM verses WHERE translation_id = ?1",
            [translation_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if already > 0 {
        return Ok(0);
    }

    let bytes = if json_bytes.starts_with(b"\xef\xbb\xbf") {
        &json_bytes[3..]
    } else {
        json_bytes
    };

    let books: Vec<ThiagoBook> =
        serde_json::from_slice(bytes).map_err(|e| format!("Bible JSON parse error: {e}"))?;

    conn.execute(
        "INSERT OR IGNORE INTO translations (id, name, abbreviation, language)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![translation_id, translation_name, abbreviation, language],
    )
    .map_err(|e| e.to_string())?;

    let map = abbrev_map();
    let mut stmt = conn
        .prepare(
            "INSERT OR IGNORE INTO verses (translation_id, book_id, chapter, verse, text)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    for book in &books {
        let abbrev = book.abbrev.to_ascii_lowercase();
        let book_id = match map.get(abbrev.as_str()) {
            Some(id) => *id,
            None => {
                eprintln!("[seeder] unknown abbrev '{abbrev}', skipping");
                continue;
            }
        };
        for (ch_idx, chapter) in book.chapters.iter().enumerate() {
            let chapter_num = (ch_idx + 1) as i32;
            for (v_idx, text) in chapter.iter().enumerate() {
                let verse_num = (v_idx + 1) as i32;
                stmt.execute(rusqlite::params![
                    translation_id,
                    book_id,
                    chapter_num,
                    verse_num,
                    text
                ])
                .map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }

    Ok(count)
}
