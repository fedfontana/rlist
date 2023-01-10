use anyhow::Result;

pub(crate) struct Topic {}

impl Topic {
    pub(crate) fn create(conn: &sqlite::Connection, topic: &impl AsRef<str>) -> Result<i64> {
        let q = "INSERT INTO topics (name) 
        VALUES (:topic) 
        ON CONFLICT (name) DO UPDATE SET name=name 
        RETURNING topic_id;";
        
        let mut stmt = conn.prepare(q)?;

        stmt.bind((":topic", topic.as_ref()))?;

        if let sqlite::State::Row = stmt.next()? {
            let topic_id = stmt.read::<i64, _>("topic_id")?;
            return Ok(topic_id);
        }

        Err(anyhow::anyhow!("There was an error creating the topic: {}", topic.as_ref()))
    }

    pub(crate) fn create_many(conn: &sqlite::Connection, topics: &Vec<impl AsRef<str>>) -> Result<Vec<i64>> {
        let q = format!(
            "INSERT INTO topics (name) 
            VALUES {} 
            ON CONFLICT (name) DO UPDATE SET name=name 
            RETURNING topic_id;",
            topics.iter().map(|_t| "(?)")
                .collect::<Vec<_>>()
                .join(", "),
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
}