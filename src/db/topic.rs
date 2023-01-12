use anyhow::Result;

use crate::read_sql_response;

pub(crate) struct DBTopic {}

impl DBTopic {
    // pub(crate) fn create(conn: &sqlite::Connection, topic: &impl AsRef<str>) -> Result<i64> {
    //     let q = "INSERT INTO topics (name) 
    //     VALUES (:topic) 
    //     ON CONFLICT (name) DO UPDATE SET name=name 
    //     RETURNING topic_id;";

    //     let mut stmt = conn.prepare(q)?;

    //     stmt.bind((":topic", topic.as_ref()))?;

    //     if let sqlite::State::Row = stmt.next()? {
    //         let topic_id = stmt.read::<i64, _>("topic_id")?;
    //         return Ok(topic_id);
    //     }

    //     Err(anyhow::anyhow!(
    //         "There was an error creating the topic: {}",
    //         topic.as_ref()
    //     ))
    // }

    /// Creates all of the topics contained in `topics`. If a topic already exists, it is ignored.
    /// Returns the list of each ot the topics' ids (in the same order as in `topics`)
    pub(crate) fn create_many(
        conn: &sqlite::Connection,
        topics: &Vec<impl AsRef<str>>,
    ) -> Result<Vec<i64>> {
        let q = format!(
            "INSERT INTO topics (name) 
            VALUES {} 
            ON CONFLICT (name) DO UPDATE SET name=name 
            RETURNING topic_id;",
            topics.iter().map(|_t| "(?)").collect::<Vec<_>>().join(", "),
        );
        let mut stmt = conn.prepare(q)?;

        stmt.bind_iter(topics.iter().enumerate().map(|(i, t)| (i + 1, t.as_ref())))?;

        let mut res = Vec::with_capacity(topics.len());

        while let sqlite::State::Row = stmt.next()? {
            let topic_id = stmt.read::<i64, _>("topic_id")?;
            res.push(topic_id);
        }

        Ok(res)
    }

    //TODO this should maybe return Ok(None) if an entry with that entry_id does not exist (?)

    /// Returns the list of tuples containing the ids and names of all of the topics related to the entry with id = `entry_id`
    pub(crate) fn get_related_to(
        conn: &sqlite::Connection,
        entry_id: i64,
    ) -> Result<Vec<(i64, String)>> {
        let q = "SELECT 
            t.name AS topic, 
            t.topic_id AS id 
        FROM topics AS t 
        JOIN rlist_has_topic AS rht 
            ON rht.topic_id = t.topic_id 
        WHERE rht.entry_id = :entry_id;";
        let mut stmt = conn.prepare(q)?;
        stmt.bind((":entry_id", entry_id))?;

        let mut res = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            read_sql_response!(stmt, id => i64, topic => String);
            res.push((id, topic));
        }

        Ok(res)
    }

    pub(crate) fn get_id_from_name(conn: &sqlite::Connection, topic: impl AsRef<str>) -> Result<i64> {
        let q = "SELECT topic_id FROM topics WHERE name = :topic;";
        let mut stmt = conn.prepare(q)?;
        stmt.bind((":topic", topic.as_ref()))?;

        if let sqlite::State::Done = stmt.next()? {
            return Err(anyhow::anyhow!("Could not find topic {} in your reading list", topic.as_ref()));
        }
        let topic_id = stmt.read::<i64, _>("topic_id")?;
        
        Ok(topic_id)
    }

    // Deletes a topic by its id. Returns None if no topic was found, else returns its name
    // pub(crate) fn delete_by_id(conn: &sqlite::Connection, topic_id: i64) -> Result<Option<String>> {
    //     let q = "DELETE FROM topics WHERE topic_id = :topic_id RETURNING *";
    //     let mut stmt = conn.prepare(q)?;
    //     stmt.bind((":topic_id", topic_id))?;

    //     if let sqlite::State::Done = stmt.next()? {
    //         return Ok(None);
    //     }

    //     let name = stmt.read::<String, _>("name")?;
    //     Ok(Some(name))
    // }
}
