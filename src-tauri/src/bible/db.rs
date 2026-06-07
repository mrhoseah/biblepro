use super::schema::create_schema;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct BibleDb(pub Mutex<Connection>);

impl BibleDb {
    pub fn open(path: &std::path::Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        create_schema(&conn)?;
        // No bundled data — all translations are downloaded or imported by the user.
        Ok(Self(Mutex::new(conn)))
    }
}
