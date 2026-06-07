use rusqlite::{Connection, Result};

pub fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS translations (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            abbreviation TEXT NOT NULL,
            language TEXT NOT NULL DEFAULT 'en'
        );

        CREATE TABLE IF NOT EXISTS books (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            short_name TEXT NOT NULL,
            testament TEXT NOT NULL CHECK(testament IN ('OT', 'NT')),
            book_order INTEGER NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS verses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            translation_id TEXT NOT NULL REFERENCES translations(id),
            book_id INTEGER NOT NULL REFERENCES books(id),
            chapter INTEGER NOT NULL,
            verse INTEGER NOT NULL,
            text TEXT NOT NULL,
            UNIQUE(translation_id, book_id, chapter, verse)
        );

        CREATE INDEX IF NOT EXISTS idx_verse_lookup
        ON verses(translation_id, book_id, chapter, verse);

        CREATE INDEX IF NOT EXISTS idx_verse_book_chapter
        ON verses(translation_id, book_id, chapter);

        CREATE VIRTUAL TABLE IF NOT EXISTS verses_fts
        USING fts5(text, translation_id UNINDEXED, content=verses, content_rowid=id);

        CREATE TRIGGER IF NOT EXISTS verses_ai AFTER INSERT ON verses BEGIN
            INSERT INTO verses_fts(rowid, text, translation_id)
            VALUES (new.id, new.text, new.translation_id);
        END;

        CREATE TRIGGER IF NOT EXISTS verses_ad AFTER DELETE ON verses BEGIN
            INSERT INTO verses_fts(verses_fts, rowid, text, translation_id)
            VALUES ('delete', old.id, old.text, old.translation_id);
        END;

        CREATE TRIGGER IF NOT EXISTS verses_au AFTER UPDATE ON verses BEGIN
            INSERT INTO verses_fts(verses_fts, rowid, text, translation_id)
            VALUES ('delete', old.id, old.text, old.translation_id);
            INSERT INTO verses_fts(rowid, text, translation_id)
            VALUES (new.id, new.text, new.translation_id);
        END;
    ",
    )?;

    // ── Study tables ──────────────────────────────────────────────────────────
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS highlights (
            book_id  INTEGER NOT NULL,
            chapter  INTEGER NOT NULL,
            verse    INTEGER NOT NULL,
            color    TEXT    NOT NULL DEFAULT 'yellow',
            PRIMARY KEY (book_id, chapter, verse)
        );

        CREATE TABLE IF NOT EXISTS notes (
            book_id    INTEGER NOT NULL,
            chapter    INTEGER NOT NULL,
            verse      INTEGER NOT NULL,
            body       TEXT    NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            PRIMARY KEY (book_id, chapter, verse)
        );

        CREATE TABLE IF NOT EXISTS bookmarks (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id    INTEGER NOT NULL,
            chapter    INTEGER NOT NULL,
            verse      INTEGER NOT NULL,
            book_name  TEXT    NOT NULL DEFAULT '',
            verse_text TEXT    NOT NULL DEFAULT '',
            label      TEXT    NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            UNIQUE(book_id, chapter, verse)
        );

        CREATE TABLE IF NOT EXISTS tags (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            name  TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#c8a45e'
        );

        CREATE TABLE IF NOT EXISTS verse_tags (
            tag_id   INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            book_id  INTEGER NOT NULL,
            chapter  INTEGER NOT NULL,
            verse    INTEGER NOT NULL,
            PRIMARY KEY (tag_id, book_id, chapter, verse)
        );

        CREATE TABLE IF NOT EXISTS study_sets (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT    NOT NULL,
            description TEXT    NOT NULL DEFAULT '',
            created_at  INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );

        CREATE TABLE IF NOT EXISTS set_verses (
            set_id    INTEGER NOT NULL REFERENCES study_sets(id) ON DELETE CASCADE,
            book_id   INTEGER NOT NULL,
            chapter   INTEGER NOT NULL,
            verse     INTEGER NOT NULL,
            book_name TEXT    NOT NULL DEFAULT '',
            verse_text TEXT   NOT NULL DEFAULT '',
            note      TEXT    NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (set_id, book_id, chapter, verse)
        );

        CREATE TABLE IF NOT EXISTS plan_progress (
            plan_id      TEXT    NOT NULL,
            day          INTEGER NOT NULL,
            completed    INTEGER NOT NULL DEFAULT 0,
            completed_at INTEGER,
            PRIMARY KEY (plan_id, day)
        );
    ",
    )?;

    seed_books(conn)?;
    Ok(())
}

fn seed_books(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    let books = canonical_books();
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO books (id, name, short_name, testament, book_order) VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;
    for (id, name, short, testament) in &books {
        stmt.execute(rusqlite::params![id, name, short, testament, id])?;
    }
    Ok(())
}

fn canonical_books() -> Vec<(i32, &'static str, &'static str, &'static str)> {
    vec![
        (1, "Genesis", "Gen", "OT"),
        (2, "Exodus", "Exo", "OT"),
        (3, "Leviticus", "Lev", "OT"),
        (4, "Numbers", "Num", "OT"),
        (5, "Deuteronomy", "Deu", "OT"),
        (6, "Joshua", "Jos", "OT"),
        (7, "Judges", "Jdg", "OT"),
        (8, "Ruth", "Rut", "OT"),
        (9, "1 Samuel", "1Sa", "OT"),
        (10, "2 Samuel", "2Sa", "OT"),
        (11, "1 Kings", "1Ki", "OT"),
        (12, "2 Kings", "2Ki", "OT"),
        (13, "1 Chronicles", "1Ch", "OT"),
        (14, "2 Chronicles", "2Ch", "OT"),
        (15, "Ezra", "Ezr", "OT"),
        (16, "Nehemiah", "Neh", "OT"),
        (17, "Esther", "Est", "OT"),
        (18, "Job", "Job", "OT"),
        (19, "Psalms", "Psa", "OT"),
        (20, "Proverbs", "Pro", "OT"),
        (21, "Ecclesiastes", "Ecc", "OT"),
        (22, "Song of Solomon", "Sol", "OT"),
        (23, "Isaiah", "Isa", "OT"),
        (24, "Jeremiah", "Jer", "OT"),
        (25, "Lamentations", "Lam", "OT"),
        (26, "Ezekiel", "Eze", "OT"),
        (27, "Daniel", "Dan", "OT"),
        (28, "Hosea", "Hos", "OT"),
        (29, "Joel", "Joe", "OT"),
        (30, "Amos", "Amo", "OT"),
        (31, "Obadiah", "Oba", "OT"),
        (32, "Jonah", "Jon", "OT"),
        (33, "Micah", "Mic", "OT"),
        (34, "Nahum", "Nah", "OT"),
        (35, "Habakkuk", "Hab", "OT"),
        (36, "Zephaniah", "Zep", "OT"),
        (37, "Haggai", "Hag", "OT"),
        (38, "Zechariah", "Zec", "OT"),
        (39, "Malachi", "Mal", "OT"),
        (40, "Matthew", "Mat", "NT"),
        (41, "Mark", "Mar", "NT"),
        (42, "Luke", "Luk", "NT"),
        (43, "John", "Joh", "NT"),
        (44, "Acts", "Act", "NT"),
        (45, "Romans", "Rom", "NT"),
        (46, "1 Corinthians", "1Co", "NT"),
        (47, "2 Corinthians", "2Co", "NT"),
        (48, "Galatians", "Gal", "NT"),
        (49, "Ephesians", "Eph", "NT"),
        (50, "Philippians", "Phi", "NT"),
        (51, "Colossians", "Col", "NT"),
        (52, "1 Thessalonians", "1Th", "NT"),
        (53, "2 Thessalonians", "2Th", "NT"),
        (54, "1 Timothy", "1Ti", "NT"),
        (55, "2 Timothy", "2Ti", "NT"),
        (56, "Titus", "Tit", "NT"),
        (57, "Philemon", "Phm", "NT"),
        (58, "Hebrews", "Heb", "NT"),
        (59, "James", "Jam", "NT"),
        (60, "1 Peter", "1Pe", "NT"),
        (61, "2 Peter", "2Pe", "NT"),
        (62, "1 John", "1Jo", "NT"),
        (63, "2 John", "2Jo", "NT"),
        (64, "3 John", "3Jo", "NT"),
        (65, "Jude", "Jud", "NT"),
        (66, "Revelation", "Rev", "NT"),
    ]
}
