use crate::entry::Entry;
use anyhow::Result;
use std::{path::Path, any};

pub struct RList {
    conn: sqlite::Connection,
}

impl RList {
    pub fn init() -> Result<Self> {
        let home_dir_path =
            dirs::home_dir().ok_or(anyhow::anyhow!("Could not find home folder"))?;
        let home_dir = Path::new(home_dir_path.as_os_str());
        let rlist_dir = home_dir.join(Path::new("rlist"));
        std::fs::create_dir_all(&rlist_dir)?;
        let p = rlist_dir.join(Path::new("rlist.sqlite"));

        let conn = sqlite::open(p)?;
        let q = "
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS rlist (
            entry_id INTEGER PRIMARY KEY,
            name TEXT NON NULL UNIQUE,
            url TEXT NOT NULL UNIQUE,
            author TEXT,
            added DATETIME NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE TABLE IF NOT EXISTS topics (
            topic_id INTEGER PRIMARY KEY,
            name TEXT NON NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS rlist_has_topic (
            entry_id INTEGER,
            topic_id INTEGER,
            PRIMARY KEY (entry_id, topic_id),
            FOREIGN KEY (entry_id) REFERENCES rlist (entry_id) ON UPDATE CASCADE ON DELETE CASCADE,
            FOREIGN KEY (topic_id) REFERENCES topics (topic_id) ON UPDATE CASCADE ON DELETE CASCADE
        );";
        conn.execute(q)?;
        Ok(Self { conn })
    }

    pub fn add(&self, entry: Entry) -> Result<bool> {
        let query = "INSERT INTO rlist (name, url, author) VALUES (:name, :url, :author) RETURNING entry_id";
        let mut statement = self.conn.prepare(query)?;
        statement.bind(
            &[
                (":name", entry.name()),
                (":url", entry.url()),
                (":author", entry.author().unwrap_or("NULL")),
            ][..],
        )?;

        let topics = entry.topics();
        if topics.len() > 0 {
            if let sqlite::State::Row = statement.next()? {
                let entry_id = statement.read::<i64, _>("entry_id")?;
                println!("Inserted id is: {entry_id}");

                let q = format!(
                    "INSERT INTO topics (name) VALUES {} 
                        ON CONFLICT (name) DO UPDATE SET name=name 
                        RETURNING topic_id;",
                    (0..topics.len())
                        .map(|e| "(?)")
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                let mut stmt = self.conn.prepare(q)?;

                stmt.bind_iter(topics.iter().enumerate().map(|(i, t)| (i + 1, t.as_str())))?;

                while let sqlite::State::Row = stmt.next()? {
                    let topic_id = stmt.read::<i64, _>("topic_id")?;
                    println!("Inserting link with topic_id:{topic_id}");
                    let q = "INSERT INTO rlist_has_topic (entry_id, topic_id) VALUES (:entry_id, :topic_id);";
                    let mut stmt = self.conn.prepare(q)?;
                    stmt.bind(&[(":entry_id", entry_id), (":topic_id", topic_id)][..])?;
                    stmt.next()?;
                }
            }
        } else {
            statement.next()?;
        }
        Ok(true)
    }

    pub fn remove_by_name(&self, name: String) -> Result<Entry> {
        let q = "DELETE FROM rlist WHERE name = :entry_name RETURNING *;";
        let mut stmt = self.conn.prepare(q)?;
        stmt.bind::<(&str, &str)>((":entry_name", name.as_ref()))?;

        if let sqlite::State::Row = stmt.next()? {
            let name = stmt.read::<String, _>("name")?;
            let url = stmt.read::<String, _>("url")?;
            let maybe_author = stmt.read::<String, _>("author")?;

            let author = if maybe_author == "NULL" {
                None
            } else {
                Some(maybe_author)
            };

            return Ok(Entry::new(name, url, author, Vec::new(), None));
        }

        Err(anyhow::anyhow!(
            "There was an error deleting the selected entry."
        ))
    }

    pub fn get_all(&self) -> Result<Vec<Entry>> {
        let q = "
            SELECT 
                ls.name AS name, 
                ls.url AS url, 
                ls.author AS author, 
                ls.added AS added, 
                t.name AS topic 
            FROM rlist AS ls 
            LEFT OUTER JOIN rlist_has_topic AS rht 
                ON ls.entry_id = rht.entry_id 
            LEFT OUTER JOIN topics AS t 
                ON t.topic_id = rht.topic_id
            ORDER BY ls.name;";
        let mut stmt = self.conn.prepare(q)?;

        let mut res: Vec<Entry> = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let name = stmt.read::<String, _>("name")?;
            let topic = stmt.read::<String, _>("topic").ok();

            let mut should_add_to_last_entry = false;
            if let Some(last) = res.last() {
                should_add_to_last_entry = last.name() == name;
            }

            if should_add_to_last_entry {
                if topic.is_some() {
                    let last = res.last_mut().expect("Checked it in the last if condition");
                    last.add_topic(topic.unwrap());
                }
            } else {
                let url = stmt.read::<String, _>("url")?;
                let maybe_author = stmt.read::<String, _>("author")?;
                let added = stmt.read::<String, _>("added")?;

                let topics = if topic.is_none() {
                    vec![]
                } else {
                    vec![topic.unwrap()]
                };

                let entry = Entry::new(
                    name.clone(),
                    url,
                    if maybe_author == "NULL" {
                        None
                    } else {
                        Some(maybe_author)
                    },
                    topics,
                    Some(added),
                );
                res.push(entry);
            }
        }
        Ok(res)
    }

    pub fn query(&self, query: String) -> Result<Vec<Entry>> {
        let q = "
            SELECT 
                ls.name AS name, 
                ls.url AS url, 
                ls.author AS author, 
                ls.added AS added, 
                t.name AS topic 
            FROM rlist AS ls 
            LEFT OUTER JOIN rlist_has_topic AS rht 
            ON ls.entry_id = rht.entry_id 
            LEFT OUTER JOIN topics AS t 
            ON t.topic_id = rht.topic_id
            WHERE ls.name LIKE '%' || :q || '%'
            ORDER BY ls.name;";
        let mut stmt = self.conn.prepare(q)?;
        stmt.bind((":q", query.as_str()))?;

        let mut res: Vec<Entry> = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let name = stmt.read::<String, _>("name")?;
            let topic = stmt.read::<String, _>("topic").ok();

            let mut should_add_to_last_entry = false;
            if let Some(last) = res.last() {
                should_add_to_last_entry = last.name() == name;
            }

            if should_add_to_last_entry {
                if topic.is_some() {
                    let last = res.last_mut().expect("Checked it in the last if condition");
                    last.add_topic(topic.unwrap());
                }
            } else {
                let url = stmt.read::<String, _>("url")?;
                let maybe_author = stmt.read::<String, _>("author")?;
                let added = stmt.read::<String, _>("added")?;

                let topics = if topic.is_none() {
                    vec![]
                } else {
                    vec![topic.unwrap()]
                };

                let entry = Entry::new(
                    name.clone(),
                    url,
                    if maybe_author == "NULL" {
                        None
                    } else {
                        Some(maybe_author)
                    },
                    topics,
                    Some(added),
                );
                res.push(entry);
            }
        }
        Ok(res)
    }

    //? how to delete topics from an entry?

    pub fn edit(
        &self,
        old_name: String,
        new_name: Option<String>,
        author: Option<String>,
        url: Option<String>,
        topics: Option<Vec<String>>,
        add_topics: Option<Vec<String>>,
        clear_topics: bool,
    ) -> Result<Entry> {
        if new_name.is_none() && author.is_none() && url.is_none() && topics.is_none() && add_topics.is_none() {
            return Err(anyhow::anyhow!("You gotta edit something, boi"));
        }

        let mut updates = Vec::new();
        let mut bindings = vec![(":old_name", old_name.as_ref())];
        if new_name.is_some() {
            updates.push("name = :new_name");
            bindings.push((":new_name", new_name.as_deref().unwrap()));
        }
        if author.is_some() {
            updates.push("author = :author");
            bindings.push((":author", author.as_deref().unwrap()));
        }
        if url.is_some() {
            updates.push("url = :url");
            bindings.push((":url", url.as_deref().unwrap()));
        }

        // if topics.is_some() {
        //     updates = format!("{updates}, name = :new_name");
        // }
        // if add_topics.is_some() {
        //     updates = format!("{updates}, name = :new_name");
        // }


        let q = format!("
                UPDATE rlist
                SET {u}
                WHERE name = :old_name
                RETURNING *;
            ",
            u = updates.join(", ")
        );

        let mut stmt = self.conn.prepare(q)?;

        stmt.bind_iter(bindings)?;

        if let sqlite::State::Row = dbg!(stmt.next())? {
            let name = stmt.read::<String, _>("name")?;
            let url = stmt.read::<String, _>("url")?;
            let maybe_author = stmt.read::<String, _>("author")?;
            let added = stmt.read::<String, _>("added")?;

            let author = if maybe_author == "NULL" {
                None
            } else {
                Some(maybe_author)
            };

            let e = Entry::new(name, url, author, vec![], Some(added));

            return Ok(e);
        }
        Err(anyhow::anyhow!("Something bad happended while updating the entry."))
    }
}
