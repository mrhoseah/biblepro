use super::schema::create_schema;
use super::seeder;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct BibleDb(pub Mutex<Connection>);

impl BibleDb {
    pub fn open(path: &std::path::Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        create_schema(&conn)?;
        if let Err(err) = seeder::seed_bundled(&conn) {
            eprintln!("[bible] bundled seed failed: {err}");
        }
        Ok(Self(Mutex::new(conn)))
    }
}
