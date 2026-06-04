use serde::{Deserialize, Serialize};
use tauri::State;
use super::db::BibleDb;

// ── types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Highlight {
    pub book_id: i32,
    pub chapter: i32,
    pub verse:   i32,
    pub color:   String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub book_id:    i32,
    pub chapter:    i32,
    pub verse:      i32,
    pub body:       String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bookmark {
    pub id:         i64,
    pub book_id:    i32,
    pub chapter:    i32,
    pub verse:      i32,
    pub book_name:  String,
    pub verse_text: String,
    pub label:      String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id:    i64,
    pub name:  String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudySet {
    pub id:          i64,
    pub name:        String,
    pub description: String,
    pub verse_count: i64,
    pub created_at:  i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SetVerse {
    pub book_id:    i32,
    pub chapter:    i32,
    pub verse:      i32,
    pub book_name:  String,
    pub verse_text: String,
    pub note:       String,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanProgress {
    pub plan_id:  String,
    pub day:      i32,
    pub completed: bool,
}

// Built-in reading plan definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanDay {
    pub day:      i32,
    pub label:    String,
    pub passages: Vec<PlanPassage>,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanPassage {
    pub book_id:   i32,
    pub book_name: String,
    pub chapter:   i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadingPlan {
    pub id:          String,
    pub name:        String,
    pub description: String,
    pub total_days:  i32,
    pub days_done:   i32,
}

// ── highlights ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_chapter_highlights(
    db: State<BibleDb>, book_id: i32, chapter: i32,
) -> Vec<Highlight> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT book_id, chapter, verse, color FROM highlights WHERE book_id=?1 AND chapter=?2"
    ).unwrap();
    stmt.query_map(rusqlite::params![book_id, chapter], |r| {
        Ok(Highlight { book_id: r.get(0)?, chapter: r.get(1)?, verse: r.get(2)?, color: r.get(3)? })
    })
    .unwrap()
    .flatten()
    .collect()
}

#[tauri::command]
pub fn set_highlight(
    db: State<BibleDb>, book_id: i32, chapter: i32, verse: i32, color: String,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO highlights (book_id, chapter, verse, color) VALUES (?1,?2,?3,?4)",
        rusqlite::params![book_id, chapter, verse, color],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_highlight(
    db: State<BibleDb>, book_id: i32, chapter: i32, verse: i32,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "DELETE FROM highlights WHERE book_id=?1 AND chapter=?2 AND verse=?3",
        rusqlite::params![book_id, chapter, verse],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ── notes ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_chapter_notes(
    db: State<BibleDb>, book_id: i32, chapter: i32,
) -> Vec<Note> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT book_id, chapter, verse, body, created_at, updated_at
         FROM notes WHERE book_id=?1 AND chapter=?2"
    ).unwrap();
    stmt.query_map(rusqlite::params![book_id, chapter], |r| {
        Ok(Note {
            book_id: r.get(0)?, chapter: r.get(1)?, verse: r.get(2)?,
            body: r.get(3)?, created_at: r.get(4)?, updated_at: r.get(5)?,
        })
    })
    .unwrap()
    .flatten()
    .collect()
}

#[tauri::command]
pub fn get_all_notes(db: State<BibleDb>) -> Vec<Note> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT book_id, chapter, verse, body, created_at, updated_at FROM notes ORDER BY updated_at DESC"
    ).unwrap();
    stmt.query_map([], |r| {
        Ok(Note {
            book_id: r.get(0)?, chapter: r.get(1)?, verse: r.get(2)?,
            body: r.get(3)?, created_at: r.get(4)?, updated_at: r.get(5)?,
        })
    })
    .unwrap()
    .flatten()
    .collect()
}

#[tauri::command]
pub fn save_note(
    db: State<BibleDb>, book_id: i32, chapter: i32, verse: i32, body: String,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    if body.trim().is_empty() {
        conn.execute(
            "DELETE FROM notes WHERE book_id=?1 AND chapter=?2 AND verse=?3",
            rusqlite::params![book_id, chapter, verse],
        ).map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO notes (book_id, chapter, verse, body)
             VALUES (?1,?2,?3,?4)
             ON CONFLICT(book_id, chapter, verse) DO UPDATE
             SET body=excluded.body, updated_at=strftime('%s','now')",
            rusqlite::params![book_id, chapter, verse, body],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── bookmarks ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_bookmarks(db: State<BibleDb>) -> Vec<Bookmark> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, book_id, chapter, verse, book_name, verse_text, label, created_at
         FROM bookmarks ORDER BY created_at DESC"
    ).unwrap();
    stmt.query_map([], |r| {
        Ok(Bookmark {
            id: r.get(0)?, book_id: r.get(1)?, chapter: r.get(2)?, verse: r.get(3)?,
            book_name: r.get(4)?, verse_text: r.get(5)?, label: r.get(6)?, created_at: r.get(7)?,
        })
    })
    .unwrap()
    .flatten()
    .collect()
}

#[tauri::command]
pub fn toggle_bookmark(
    db: State<BibleDb>,
    book_id: i32, chapter: i32, verse: i32,
    book_name: String, verse_text: String,
) -> Result<bool, String> {
    let conn = db.0.lock().unwrap();
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bookmarks WHERE book_id=?1 AND chapter=?2 AND verse=?3",
        rusqlite::params![book_id, chapter, verse],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;

    if exists > 0 {
        conn.execute(
            "DELETE FROM bookmarks WHERE book_id=?1 AND chapter=?2 AND verse=?3",
            rusqlite::params![book_id, chapter, verse],
        ).map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        conn.execute(
            "INSERT INTO bookmarks (book_id, chapter, verse, book_name, verse_text) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![book_id, chapter, verse, book_name, verse_text],
        ).map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub fn delete_bookmark(db: State<BibleDb>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM bookmarks WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn is_bookmarked(db: State<BibleDb>, book_id: i32, chapter: i32, verse: i32) -> bool {
    let conn = db.0.lock().unwrap();
    conn.query_row(
        "SELECT COUNT(*) FROM bookmarks WHERE book_id=?1 AND chapter=?2 AND verse=?3",
        rusqlite::params![book_id, chapter, verse],
        |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0
}

// ── tags ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_tags(db: State<BibleDb>) -> Vec<Tag> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY name").unwrap();
    stmt.query_map([], |r| Ok(Tag { id: r.get(0)?, name: r.get(1)?, color: r.get(2)? }))
        .unwrap().flatten().collect()
}

#[tauri::command]
pub fn create_tag(db: State<BibleDb>, name: String, color: String) -> Result<i64, String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?1,?2)",
        rusqlite::params![name, color],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn tag_verse(
    db: State<BibleDb>, tag_id: i64, book_id: i32, chapter: i32, verse: i32,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO verse_tags (tag_id, book_id, chapter, verse) VALUES (?1,?2,?3,?4)",
        rusqlite::params![tag_id, book_id, chapter, verse],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn untag_verse(
    db: State<BibleDb>, tag_id: i64, book_id: i32, chapter: i32, verse: i32,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "DELETE FROM verse_tags WHERE tag_id=?1 AND book_id=?2 AND chapter=?3 AND verse=?4",
        rusqlite::params![tag_id, book_id, chapter, verse],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_verse_tags(db: State<BibleDb>, book_id: i32, chapter: i32, verse: i32) -> Vec<Tag> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color FROM tags t
         JOIN verse_tags vt ON t.id=vt.tag_id
         WHERE vt.book_id=?1 AND vt.chapter=?2 AND vt.verse=?3"
    ).unwrap();
    stmt.query_map(rusqlite::params![book_id, chapter, verse], |r| {
        Ok(Tag { id: r.get(0)?, name: r.get(1)?, color: r.get(2)? })
    }).unwrap().flatten().collect()
}

// ── study sets ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_study_sets(db: State<BibleDb>) -> Vec<StudySet> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.description, COUNT(v.set_id) as vc, s.created_at
         FROM study_sets s
         LEFT JOIN set_verses v ON s.id=v.set_id
         GROUP BY s.id ORDER BY s.created_at DESC"
    ).unwrap();
    stmt.query_map([], |r| {
        Ok(StudySet {
            id: r.get(0)?, name: r.get(1)?, description: r.get(2)?,
            verse_count: r.get(3)?, created_at: r.get(4)?,
        })
    }).unwrap().flatten().collect()
}

#[tauri::command]
pub fn create_study_set(db: State<BibleDb>, name: String, description: String) -> Result<i64, String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO study_sets (name, description) VALUES (?1,?2)",
        rusqlite::params![name, description],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_study_set(db: State<BibleDb>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM study_sets WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_set_verses(db: State<BibleDb>, set_id: i64) -> Vec<SetVerse> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT book_id, chapter, verse, book_name, verse_text, note, sort_order
         FROM set_verses WHERE set_id=?1 ORDER BY sort_order, book_id, chapter, verse"
    ).unwrap();
    stmt.query_map(rusqlite::params![set_id], |r| {
        Ok(SetVerse {
            book_id: r.get(0)?, chapter: r.get(1)?, verse: r.get(2)?,
            book_name: r.get(3)?, verse_text: r.get(4)?, note: r.get(5)?, sort_order: r.get(6)?,
        })
    }).unwrap().flatten().collect()
}

#[tauri::command]
pub fn add_to_study_set(
    db: State<BibleDb>, set_id: i64,
    book_id: i32, chapter: i32, verse: i32,
    book_name: String, verse_text: String,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let max_order: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order),0) FROM set_verses WHERE set_id=?1",
        rusqlite::params![set_id], |r| r.get(0),
    ).unwrap_or(0);
    conn.execute(
        "INSERT OR IGNORE INTO set_verses (set_id, book_id, chapter, verse, book_name, verse_text, sort_order)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        rusqlite::params![set_id, book_id, chapter, verse, book_name, verse_text, max_order + 1],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_set_verse_note(
    db: State<BibleDb>, set_id: i64,
    book_id: i32, chapter: i32, verse: i32, note: String,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE set_verses SET note=?1 WHERE set_id=?2 AND book_id=?3 AND chapter=?4 AND verse=?5",
        rusqlite::params![note, set_id, book_id, chapter, verse],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_from_study_set(
    db: State<BibleDb>, set_id: i64, book_id: i32, chapter: i32, verse: i32,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "DELETE FROM set_verses WHERE set_id=?1 AND book_id=?2 AND chapter=?3 AND verse=?4",
        rusqlite::params![set_id, book_id, chapter, verse],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// Export a study set as JSON string
#[tauri::command]
pub fn export_study_set(db: State<BibleDb>, id: i64) -> Result<String, String> {
    let conn = db.0.lock().unwrap();
    let set: Option<(String, String)> = conn.query_row(
        "SELECT name, description FROM study_sets WHERE id=?1",
        rusqlite::params![id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).ok();
    let (name, description) = set.ok_or("Set not found")?;

    let mut stmt = conn.prepare(
        "SELECT book_id, chapter, verse, book_name, verse_text, note FROM set_verses WHERE set_id=?1 ORDER BY sort_order"
    ).map_err(|e| e.to_string())?;
    let verses: Vec<serde_json::Value> = stmt.query_map(rusqlite::params![id], |r| {
        Ok(serde_json::json!({
            "book_id": r.get::<_,i32>(0)?, "chapter": r.get::<_,i32>(1)?,
            "verse": r.get::<_,i32>(2)?, "book_name": r.get::<_,String>(3)?,
            "verse_text": r.get::<_,String>(4)?, "note": r.get::<_,String>(5)?,
        }))
    }).map_err(|e| e.to_string())?
    .flatten().collect();

    let export = serde_json::json!({ "name": name, "description": description, "verses": verses, "version": 1 });
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

// ── reading plans ─────────────────────────────────────────────────────────────

pub fn built_in_plans() -> Vec<(&'static str, &'static str, &'static str, i32)> {
    // (id, name, description, total_days)
    vec![
        ("nt90",    "New Testament in 90 Days",       "Read the entire NT in 3 months, ~3 chapters/day.", 90),
        ("psalms30","Psalms in 30 Days",               "All 150 Psalms in one month, 5 per day.",          30),
        ("proverbs","Proverbs in a Month",             "One chapter of Proverbs per day for 31 days.",     31),
        ("genesis", "Genesis in 7 Days",               "The first book of the Bible in a week.",            7),
        ("sermon",  "Sermon on the Mount (5 Days)",   "Matthew 5–7, the heart of Jesus' teaching.",        5),
    ]
}

#[tauri::command]
pub fn get_reading_plans(db: State<BibleDb>) -> Vec<ReadingPlan> {
    let conn = db.0.lock().unwrap();
    built_in_plans().into_iter().map(|(id, name, desc, days)| {
        let done: i32 = conn.query_row(
            "SELECT COUNT(*) FROM plan_progress WHERE plan_id=?1 AND completed=1",
            rusqlite::params![id], |r| r.get(0),
        ).unwrap_or(0);
        ReadingPlan {
            id: id.to_string(), name: name.to_string(), description: desc.to_string(),
            total_days: days, days_done: done,
        }
    }).collect()
}

#[tauri::command]
pub fn get_plan_days(db: State<BibleDb>, plan_id: String) -> Vec<PlanDay> {
    let conn = db.0.lock().unwrap();
    let passages = plan_passages(&plan_id);
    passages.into_iter().map(|(day, label, refs)| {
        let completed: bool = conn.query_row(
            "SELECT completed FROM plan_progress WHERE plan_id=?1 AND day=?2",
            rusqlite::params![plan_id, day], |r| r.get::<_, i32>(0),
        ).unwrap_or(0) == 1;
        PlanDay {
            day, label,
            passages: refs.into_iter().map(|(book_id, book_name, chapter)| {
                PlanPassage { book_id, book_name: book_name.to_string(), chapter }
            }).collect(),
            completed,
        }
    }).collect()
}

#[tauri::command]
pub fn mark_plan_day(
    db: State<BibleDb>, plan_id: String, day: i32, completed: bool,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let ts: Option<i64> = if completed { Some(chrono_now()) } else { None };
    conn.execute(
        "INSERT INTO plan_progress (plan_id, day, completed, completed_at)
         VALUES (?1,?2,?3,?4)
         ON CONFLICT(plan_id, day) DO UPDATE SET completed=excluded.completed, completed_at=excluded.completed_at",
        rusqlite::params![plan_id, day, completed as i32, ts],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// Hard-coded plan schedules: (day, label, [(book_id, book_name, chapter)])
fn plan_passages(plan_id: &str) -> Vec<(i32, String, Vec<(i32, &'static str, i32)>)> {
    match plan_id {
        "nt90" => nt_90_days(),
        "psalms30" => psalms_30_days(),
        "proverbs" => proverbs_31_days(),
        "genesis" => genesis_7_days(),
        "sermon" => sermon_5_days(),
        _ => vec![],
    }
}

fn genesis_7_days() -> Vec<(i32, String, Vec<(i32, &'static str, i32)>)> {
    vec![
        (1, "Creation".into(), vec![(1,"Genesis",1),(1,"Genesis",2),(1,"Genesis",3),(1,"Genesis",4),(1,"Genesis",5),(1,"Genesis",6)]),
        (2, "The Flood".into(), vec![(1,"Genesis",7),(1,"Genesis",8),(1,"Genesis",9),(1,"Genesis",10),(1,"Genesis",11)]),
        (3, "Abraham".into(), vec![(1,"Genesis",12),(1,"Genesis",13),(1,"Genesis",14),(1,"Genesis",15),(1,"Genesis",16),(1,"Genesis",17)]),
        (4, "Isaac & Jacob".into(), vec![(1,"Genesis",18),(1,"Genesis",19),(1,"Genesis",20),(1,"Genesis",21),(1,"Genesis",22),(1,"Genesis",23),(1,"Genesis",24)]),
        (5, "Jacob's Journey".into(), vec![(1,"Genesis",25),(1,"Genesis",26),(1,"Genesis",27),(1,"Genesis",28),(1,"Genesis",29),(1,"Genesis",30)]),
        (6, "Joseph".into(), vec![(1,"Genesis",31),(1,"Genesis",32),(1,"Genesis",33),(1,"Genesis",34),(1,"Genesis",35),(1,"Genesis",36),(1,"Genesis",37),(1,"Genesis",38),(1,"Genesis",39),(1,"Genesis",40)]),
        (7, "Joseph & Egypt".into(), vec![(1,"Genesis",41),(1,"Genesis",42),(1,"Genesis",43),(1,"Genesis",44),(1,"Genesis",45),(1,"Genesis",46),(1,"Genesis",47),(1,"Genesis",48),(1,"Genesis",49),(1,"Genesis",50)]),
    ]
}

fn sermon_5_days() -> Vec<(i32, String, Vec<(i32, &'static str, i32)>)> {
    vec![
        (1, "Beatitudes & Salt and Light".into(), vec![(40,"Matthew",5)]),
        (2, "Prayer & Fasting".into(), vec![(40,"Matthew",6)]),
        (3, "Do Not Worry".into(), vec![(40,"Matthew",6)]),
        (4, "Judge Not".into(), vec![(40,"Matthew",7)]),
        (5, "The Two Foundations".into(), vec![(40,"Matthew",7)]),
    ]
}

fn psalms_30_days() -> Vec<(i32, String, Vec<(i32, &'static str, i32)>)> {
    (1..=30).map(|day| {
        let start = (day - 1) * 5 + 1;
        let passages = (start..=start+4)
            .filter(|&c| c <= 150)
            .map(|c| (19i32, "Psalms", c))
            .collect();
        (day, format!("Psalms {start}–{}", start+4), passages)
    }).collect()
}

fn proverbs_31_days() -> Vec<(i32, String, Vec<(i32, &'static str, i32)>)> {
    (1..=31).map(|day| {
        (day, format!("Proverbs {day}"), vec![(20i32, "Proverbs", day)])
    }).collect()
}

fn nt_90_days() -> Vec<(i32, String, Vec<(i32, &'static str, i32)>)> {
    // NT books: Matthew(40)=28ch, Mark(41)=16, Luke(42)=24, John(43)=21,
    // Acts(44)=28, Rom(45)=16, 1Co(46)=16, 2Co(47)=13, Gal(48)=6, Eph(49)=6,
    // Phil(50)=4, Col(51)=4, 1Th(52)=5, 2Th(53)=3, 1Ti(54)=6, 2Ti(55)=4,
    // Tit(56)=3, Phm(57)=1, Heb(58)=13, Jam(59)=5, 1Pe(60)=5, 2Pe(61)=3,
    // 1Jo(62)=5, 2Jo(63)=1, 3Jo(64)=1, Jud(65)=1, Rev(66)=22
    let all: Vec<(i32, &'static str, i32)> = vec![
        (40,"Matthew",28),(41,"Mark",16),(42,"Luke",24),(43,"John",21),
        (44,"Acts",28),(45,"Romans",16),(46,"1 Corinthians",16),(47,"2 Corinthians",13),
        (48,"Galatians",6),(49,"Ephesians",6),(50,"Philippians",4),(51,"Colossians",4),
        (52,"1 Thessalonians",5),(53,"2 Thessalonians",3),(54,"1 Timothy",6),(55,"2 Timothy",4),
        (56,"Titus",3),(57,"Philemon",1),(58,"Hebrews",13),(59,"James",5),
        (60,"1 Peter",5),(61,"2 Peter",3),(62,"1 John",5),(63,"2 John",1),
        (64,"3 John",1),(65,"Jude",1),(66,"Revelation",22),
    ];

    // Flatten all chapters in order
    let chapters: Vec<(i32, &'static str, i32)> = all.into_iter()
        .flat_map(|(book_id, name, ch_count)| (1..=ch_count).map(move |c| (book_id, name, c)))
        .collect();

    // 3 chapters per day for 90 days
    chapters.chunks(3).enumerate()
        .map(|(i, chunk)| {
            let day = i as i32 + 1;
            let label = if chunk.len() == 1 {
                format!("{} {}", chunk[0].1, chunk[0].2)
            } else {
                format!("{} {}–{}", chunk[0].1, chunk[0].2, chunk.last().unwrap().2)
            };
            (day, label, chunk.to_vec())
        })
        .collect()
}
