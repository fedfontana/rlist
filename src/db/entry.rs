use anyhow::Result;

use crate::db::topic::DBTopic;
use crate::entry::Entry;
use crate::read_sql_response;
use crate::utils::{opt_from_sql, ToSQL};

pub struct DBEntry {}

impl DBEntry {
    pub(crate) fn associate_with_topics(
        conn: &sqlite::Connection,
        entry_id: i64,
        topic_ids: Vec<i64>,
    ) -> Result<()> {
        let q = format!(
            "INSERT INTO rlist_has_topic (entry_id, topic_id) VALUES {} 
            ON CONFLICT (entry_id, topic_id) DO UPDATE SET entry_id=entry_id;",
            topic_ids
                .iter()
                .map(|_t| "(?, ?)")
                .collect::<Vec<_>>()
                .join(", ")
        );

        let mut stmt = conn.prepare(q)?;
        let bindings = (0..2 * topic_ids.len())
            .map(|idx| {
                if idx % 2 == 0 {
                    (idx + 1, entry_id)
                } else {
                    (idx + 1, topic_ids[(idx - 1) / 2])
                }
            })
            .collect::<Vec<_>>();

        stmt.bind(bindings.as_slice())?;
        stmt.next()?;

        Ok(())
    }

    pub(crate) fn create(
        conn: &sqlite::Connection,
        name: &str,
        url: &str,
        author: Option<&str>,
    ) -> Result<i64> {
        let q= "INSERT INTO rlist (name, url, author) VALUES (:name, :url, :author) RETURNING entry_id";
        let mut stmt = conn.prepare(q)?;
        stmt.bind(&[(":name", name), (":url", url), (":author", author.to_sql().as_ref())][..])?;

        if let sqlite::State::Done = stmt.next()? {
            return Err(anyhow::anyhow!("Could not insert entry with name: {name}"));
        }

        let entry_id = stmt.read::<i64, _>("entry_id")?;
        Ok(entry_id)
    }

    //TODO maybe i should just return an Err("Not found") instead of Ok(None)
    //? is it possible to write a subquery in the RETURNING clause? 
    //? if yes, then i could also return all of the topics from the delete clause?
    pub(crate) fn remove_by_name(
        conn: &sqlite::Connection,
        name: impl AsRef<str>,
    ) -> Result<Option<Entry>> {
        let entry_id = Self::get_id_from_name(conn, name.as_ref())?;
        // Early return if no result is found
        if entry_id.is_none() {
            return Ok(None);
        }
        let entry_id = entry_id.unwrap();

        let topics = DBTopic::get_related_to(conn, entry_id)?
            .into_iter()
            .map(|(_i, t)| t)
            .collect::<Vec<_>>();
        
        let q = "DELETE FROM rlist WHERE name = :entry_name RETURNING *;";
        let mut stmt = conn.prepare(q)?;
        stmt.bind((":entry_name", name.as_ref()))?;
        
        if let sqlite::State::Done = stmt.next()? {
            return Ok(None);
        }

        read_sql_response!(stmt, name => String, url => String, added => String, author => String);
        let author = opt_from_sql(author);

        return Ok(Some(Entry::new(
            name,
            url,
            author,
            topics,
            Some(added),
        )));
    }

    pub(crate) fn get_id_from_name(conn: &sqlite::Connection, name: impl AsRef<str>) -> Result<Option<i64>> {
        let q = "SELECT entry_id FROM rlist WHERE name=:name;";
        let mut stmt = conn.prepare(q)?;
        stmt.bind((":name", name.as_ref()))?;
        if let sqlite::State::Done =  stmt.next()? {
            return Ok(None);
        }
        let entry_id = stmt.read::<i64, _>("entry_id")?;
        Ok(Some(entry_id))
    }

    pub(crate) fn unlink_all_topics(conn: &sqlite::Connection, entry_id: i64) -> Result<()> {
        let q = 
            "DELETE FROM rlist_has_topic 
                    WHERE entry_id = :entry_id;";

        let mut stmt = conn.prepare(q)?;

        stmt.bind((":entry_id", entry_id))?;
        stmt.next()?;

        Ok(())
    }

    pub(crate) fn unlink_topics_by_name(conn: &sqlite::Connection, entry_id: i64, topics: Vec<String>) -> Result<()> {
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

    pub(crate) fn get_by_name_without_topics(conn: &sqlite::Connection, name: impl AsRef<str>) -> Result<(i64, crate::Entry)> {
        let q = "SELECT * FROM rlist WHERE name = :name;";
        let mut stmt = conn.prepare(q)?;
        stmt.bind((":name", name.as_ref()))?;

        if let sqlite::State::Done = stmt.next()? {
            return Err(anyhow::anyhow!("Could not find rlist entry with name: {}", name.as_ref()));
        }

        read_sql_response!(stmt, entry_id => i64, name => String, url => String, added => String, author => String);
        let author = opt_from_sql(author);
        
        Ok((entry_id, Entry::new(name, url, author, Vec::new(), Some(added))))
    }
}