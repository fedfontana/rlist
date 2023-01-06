use crate::entry::Entry;
use anyhow::Result;

pub struct RList {
    conn: sqlite::Connection,
}

impl RList {
    pub fn init() -> Result<Self> {
        let conn = sqlite::open("rlist.sqlite")?;
        let q = "CREATE TABLE IF NOT EXISTS rlist (
            entry_id INTEGER PRIMARY KEY,
            name TEXT NON NULL UNIQUE,
            url TEXT NOT NULL UNIQUE,
            author TEXT,
            added DATETIME NOT NULL DEFAULT (datetime('now', 'localtime'))
        );";
        conn.execute(q)?;
        Ok(Self { conn })
    }

    pub fn add(&self, entry: Entry) -> Result<bool> {
        let query = "INSERT INTO rlist (name, url, author) VALUES (:name, :url, :author)";
        let mut statement = self.conn.prepare(query)?;
        statement.bind(
            &[
                (":name", entry.name()),
                (":url", entry.url()),
                (":author", entry.author().unwrap_or("NULL")),
            ][..],
        )?;
        statement.next()?;
        //while let Ok(sqlite::State::Row) = statement.next() {}
        Ok(true)

        // if self.content.iter().position(|e| e.id() == new_entry.id()).is_some() {
        //     return false;
        // }
        // self.content.push(new_entry);
        // true
    }

    pub fn remove_with_id(&mut self, id: impl AsRef<str>) -> Option<Entry> {
        // if let Some(idx) = self.content.iter().position(|e| e.id() == id.as_ref()) {
        //     Some(self.content.remove(idx))
        // } else {
        //     None
        // }
        None
    }
}
