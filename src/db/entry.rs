use anyhow::Result;
use colored::Colorize;
use rusqlite::OptionalExtension;

use crate::entry::Entry;
use crate::read_sql_response;
use crate::utils::opt_from_sql;

use super::Table;

impl Table for Entry {
    fn schema() -> &'static str {
        "CREATE TABLE IF NOT EXISTS entry (
            id TEXT PRIMARY KEY,
            name TEXT NON NULL UNIQUE,
            url TEXT NOT NULL UNIQUE,
            author TEXT,
            added DATETIME NOT NULL DEFAULT (datetime('now', 'localtime'))
        );"
    }

    fn persist_changes(&self, conn: &mut rusqlite::Connection) -> rusqlite::Result<()> {
        todo!()
    }

    fn delete(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        todo!()
    }
}

impl Entry {
    /// Associates the entry identified by `entry_id` to all of the topics identified by `topic_ids`
    // pub(crate) fn associate_with_topics(
    //     conn: &rusqlite::Connection,
    //     entry_id: i64,
    //     topic_ids: Vec<i64>,
    // ) -> Result<()> {
    //     let q = format!(
    //         "INSERT INTO rlist_has_topic (entry_id, topic_id) VALUES {}
    //         ON CONFLICT (entry_id, topic_id) DO UPDATE SET entry_id=entry_id;",
    //         topic_ids
    //             .iter()
    //             .map(|_t| "(?, ?)")
    //             .collect::<Vec<_>>()
    //             .join(", ")
    //     );
    //
    //     let bindings = (0..2 * topic_ids.len()).map(|idx| {
    //         if idx % 2 == 0 {
    //             entry_id
    //         } else {
    //             topic_ids[(idx - 1) / 2]
    //         }
    //     });
    //
    //     conn.execute(&q, rusqlite::params_from_iter(bindings))?;
    //
    //     Ok(())
    // }

    /// Creates a new entry in the db. Does not handle topics. Returns a tuple containing the entry_id and the entry
    /// Expects added to be formatted like SQLITE_DATETIME_FORMAT
    // pub(crate) fn create(&mut self, conn: &rusqlite::Connection) -> Result<(i64, Entry)> {
    //     let q = conn.prepare("INSERT INTO rlist (name, url, author, added) VALUES (:name, :url, :author, :added) RETURNING *")?;
    //
    // q.query_row(
    //         rusqlite::named_params! {
    //                 ":name": name,
    //                 ":url": url,
    //                 ":author": author,
    //                 ":added": added.to_string(), // expected to be in the right format
    //         },
    //
    //         |r| {
    //     Ok((
    //         r.get::<_, i64>(1)?,
    //         Entry::new(
    //             name.to_string(),
    //             url.to_string(),
    //             author.map(|s| s.into()),
    //             vec![],
    //             Some(r.get::<_, String>(5)?),
    //         )
    //     ))
    //
    //
    // )
    //
    // Ok(q.query_row(params,
    // })?)
    // }

    //? is it possible to write a subquery in the RETURNING clause to return all of the topics instead of doing 2 queries?
    /// Removes the entry with name = `name`.
    /// Returns the old entry's data with all of its topic
    pub(crate) fn remove_by_name(
        conn: &rusqlite::Connection,
        name: impl AsRef<str>,
    ) -> rusqlite::Result<Entry> {
        // let entry_id = Self::get_id_from_name(conn, name.as_ref())?;
        // let entry_id = entry_id.ok_or(anyhow::anyhow!(
        //     "Could not find any entry with name {} in your reading list",
        //     name.as_ref().bold().truecolor(255, 165, 0)
        // ))?;
        //
        // let topics = DBTopic::get_related_to(conn, entry_id)?
        //     .into_iter()
        //     .map(|(_i, t)| t)
        //     .collect::<Vec<_>>();
        //
        // let q = "DELETE FROM rlist WHERE name = :entry_name RETURNING *;";
        // let mut stmt = conn.prepare(q)?;
        // stmt.bind((":entry_name", name.as_ref()))?;
        // // No need to check it is == State::Done since i already check that it exists with Self::get_id_from_name()
        // stmt.next()?;
        //
        // read_sql_response!(stmt, name => String, url => String, added => String, author => String);
        // let author = opt_from_sql(author);
        //
        // Ok(Entry::new(name, url, author, topics, Some(added)))

        Ok(())
    }

    /// Gets an entry_id given a name.
    /// Returns None if no entry with that name was found.
    pub(crate) fn get_id_from_name(
        conn: &rusqlite::Connection,
        name: impl AsRef<str>,
    ) -> rusqlite::Result<Option<i64>> {
        let mut stmt = conn.prepare("SELECT entry_id FROM rlist WHERE name=:name;")?;

        stmt.query_row(rusqlite::named_params! { ":name": name.as_ref() }, |r| {
            r.get::<_, i64>(1)
        })
        .optional()
    }

    /// Removes the entry with `entry_id` from all of its topics.
    pub(crate) fn unlink_all_topics(
        conn: &rusqlite::Connection,
        entry_id: i64,
    ) -> rusqlite::Result<()> {
        conn.execute(
            "DELETE FROM rlist_has_topic WHERE entry_id = :entry_id;",
            rusqlite::named_params! { ":entry_id": entry_id },
        )
        .map(|_| ())
    }

    /// Removes the entry with id = `entry_id` from all of the topics in `topics`
    pub(crate) fn unlink_topics_by_name(
        conn: &sqlite::Connection,
        entry_id: i64,
        topics: Vec<String>,
    ) -> Result<()> {
        let q = format!(
            "DELETE FROM rlist_has_topic 
                    WHERE entry_id = ?
                        AND topic_id IN (
                            SELECT topic_id FROM topics WHERE name IN ({})
                    ) RETURNING *;",
            (0..topics.len())
                .map(|_e| "?")
                .collect::<Vec<_>>()
                .join(", "),
        );

        let mut stmt = conn.prepare(q)?;

        let bindings = [(1, sqlite::Value::from(entry_id))].into_iter().chain(
            topics
                .iter()
                .enumerate()
                .map(|(i, t)| (i + 2, sqlite::Value::from(t.as_str()))),
        );

        stmt.bind_iter(bindings)?;
        stmt.next()?;

        Ok(())
    }

    /// Returns the tuple (entry_id, Entry) containing the entry with name = `name`
    pub(crate) fn get_by_name_without_topics(
        conn: &sqlite::Connection,
        name: impl AsRef<str>,
    ) -> Result<(i64, crate::Entry)> {
        let q = "SELECT * FROM rlist WHERE name = :name;";
        let mut stmt = conn.prepare(q)?;
        stmt.bind((":name", name.as_ref()))?;

        if let sqlite::State::Done = stmt.next()? {
            return Err(anyhow::anyhow!(
                "Could not find any entry in your reading list with name {}",
                name.as_ref().bold().truecolor(255, 165, 0)
            ));
        }

        read_sql_response!(stmt, entry_id => i64, name => String, url => String, added => String, author => String);
        let author = opt_from_sql(author);

        Ok((
            entry_id,
            Entry::new(name, url, author, Vec::new(), Some(added)),
        ))
    }

    /// Returns all entries with all of their topics
    pub(crate) fn get_all_complete(conn: &sqlite::Connection) -> Result<Vec<Entry>> {
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
            ON t.topic_id = rht.topic_id;";

        let mut stmt = conn.prepare(q)?;

        let mut res: Vec<Entry> = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let name = stmt.read::<String, _>("name")?;
            let topic = stmt.read::<String, _>("topic").ok();

            if let Some(pos) = res.iter().position(|e| e.name == name) {
                if topic.is_some() {
                    res[pos].topics.push(topic.unwrap());
                }
            } else {
                read_sql_response!(stmt, url => String, added => String, author => String);
                let author = opt_from_sql(author);

                let topics = topic.map(|t| vec![t]).unwrap_or_default();

                let entry = Entry::new(name.clone(), url, author, topics, Some(added));
                res.push(entry);
            }
        }
        Ok(res)
    }

    pub(crate) fn remove_related_to(conn: &sqlite::Connection, topic_id: i64) -> Result<()> {
        let q = "DELETE FROM rlist 
        WHERE entry_id IN (
            SELECT entry_id 
            FROM rlist_has_topic 
            WHERE topic_id = :topic_id
        );";

        let mut stmt = conn.prepare(q)?;
        stmt.bind((":topic_id", topic_id))?;
        stmt.next()?;

        Ok(())
    }
}

// TODO: should probably also populate the topics list before deleting them with cascade
pub(crate) fn remove_entries_by_name(
    conn: &rusqlite::Connection,
    name: &str,
) -> rusqlite::Result<Vec<Entry>> {
    let mut q = conn.prepare(
        "DELETE FROM entry 
        WHERE name = :name
        RETURNING name, url, author",
    )?;

    q.query_map(
        rusqlite::named_params! { ":name": name },
        |r| -> rusqlite::Result<Entry> {
            Ok(Entry::new(
                r.get::<_, String>(1)?,         // Name
                r.get::<_, String>(2)?,         // Url
                r.get::<_, Option<String>>(3)?, // Author
            ))
        },
    )
    .collect::<rusqlite::Result<Vec<Entry>>>()
}
