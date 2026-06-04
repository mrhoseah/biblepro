use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Translation {
    pub id: String,
    pub name: String,
    pub abbreviation: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book {
    pub id: i32,
    pub name: String,
    pub short_name: String,
    pub testament: String,
    pub book_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Verse {
    pub id: i64,
    pub translation_id: String,
    pub book_id: i32,
    pub book_name: String,
    pub chapter: i32,
    pub verse: i32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub verse: Verse,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterInfo {
    pub book_name: String,
    pub book_id: i32,
    pub chapter: i32,
    pub total_chapters: i32,
    pub verses: Vec<Verse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbStats {
    pub translation_count: i64,
    pub book_count: i64,
    pub verse_count: i64,
    pub translations: Vec<Translation>,
}
