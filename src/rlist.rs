use crate::{entry::Entry, topic::Topic};
use anyhow::Result;
use chrono::DateTime;
use dateparser::DateTimeUtc;
use std::{any, collections::HashSet, fmt::Display, path::Path, str::FromStr};

use crate::read_sql_response;
use crate::utils::{opt_from_sql, dt_to_string};
use crate::db::{topic::DBTopic, entry::DBEntry};

#[derive(Debug, Clone)]
pub enum OrderBy {
    Name,
    Url,
    Author,
    Added,
}

impl FromStr for OrderBy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "name" => Ok(Self::Name),
            "url" => Ok(Self::Url),
            "author" => Ok(Self::Author),
            "added" => Ok(Self::Added),
            other => Err(anyhow::anyhow!("Option not recognized")),
        }
    }
}

impl ToString for OrderBy {
    fn to_string(&self) -> String {
        (match self {
            OrderBy::Name => "name",
            OrderBy::Url => "url",
            OrderBy::Author => "author",
            OrderBy::Added => "added",
        })
        .to_string()
    }
}

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
        let entry_id = DBEntry::create(&self.conn, entry.name(), entry.url(), entry.author())?;

        let topics = entry.topics();
        if topics.len() > 0 {
            let topic_ids = DBTopic::create_many(&self.conn, topics)?;
            DBEntry::associate_with_topics(&self.conn, entry_id, topic_ids)?;
        }

        Ok(true)
    }

    pub fn remove_by_name(&self, name: String) -> Result<Entry> {
        let r = DBEntry::remove_by_name(&self.conn, name.clone())?;
        if r.is_none() {
            return Err(anyhow::anyhow!("No entry found with name: {name}"));
        }
        Ok(r.unwrap())
    }

    pub fn query(
        &self,
        query: Option<String>,
        topics: Option<Vec<String>>,
        author: Option<String>,
        url: Option<String>,
        sort_by: Option<OrderBy>,
        desc: bool,
        from: Option<DateTimeUtc>,
        to: Option<DateTimeUtc>,
    ) -> Result<Vec<Entry>> {
        //TODO maybe sort NON NULLS first? like if used with `-s author`, put the ones with authors first (sorted by author),
        //TODO and then the ones with author == NULL

        let mut bindings = Vec::new();
        let mut clauses = Vec::new();
        if query.is_some() {
            clauses.push("ls.name LIKE '%' || :q || '%'");
            bindings.push((":q", query.as_deref().unwrap()));
        };
        if author.is_some() {
            clauses.push("ls.author LIKE '%' || :author || '%'");
            bindings.push((":author", author.as_deref().unwrap()));
        }
        if url.is_some() {
            clauses.push("ls.url LIKE '%' || :url || '%'");
            bindings.push((":url", url.as_deref().unwrap()));
        }

        // SQLite format:  YYYY-MM-DD HH:MM:SS
        let opt_from = from.map(|dt| dt_to_string(dt));
        if let Some(from) = opt_from.as_deref() {
            clauses.push("ls.added >= :from");
            bindings.push((":from", from.as_ref()));
        }
        let opt_to = to.map(|dt| dt_to_string(dt));
        if let Some(to) = opt_to.as_deref() {
            clauses.push("ls.added <= :to");
            bindings.push((":to", to.as_ref()));
        }

        let sort = if let Some(sort_col) = sort_by {
            let order = if desc { "DESC" } else { "ASC" };
            format!("ORDER BY {} {};", sort_col.to_string(), order)
        } else {
            ";".to_string()
        };

        let q = format!(
            "
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
            {}
            {sort}",
            if clauses.len() > 0 {
                format!("WHERE {}", clauses.join(" AND "))
            } else {
                "".to_string()
            }
        );

        let mut stmt = self.conn.prepare(q)?;
        stmt.bind_iter(bindings)?;

        let mut res: Vec<Entry> = Vec::new();

        while let sqlite::State::Row = stmt.next()? {
            let name = stmt.read::<String, _>("name")?;
            let topic = stmt.read::<String, _>("topic").ok();

            if let Some(pos) = res.iter().position(|e| e.name() == name) {
                if topic.is_some() {
                    res[pos].add_topic(topic.clone().unwrap());
                }
            } else {
                read_sql_response!(stmt, url => String, added => String, author => String);
                let author = opt_from_sql(author);

                let topics = topic.map(|t| vec![t]).unwrap_or_default();

                let entry = Entry::new(
                    name.clone(),
                    url,
                    author,
                    topics,
                    Some(added),
                );
                res.push(entry);
            }
        }

        if let Some(topics) = topics {
            let required_topics_set = topics.iter().collect::<HashSet<_>>();

            res = res
                .into_iter()
                .filter(|entry| {
                    let entry_topics_set = entry.topics().iter().collect::<HashSet<_>>();

                    entry_topics_set
                        .intersection(&required_topics_set)
                        .collect::<Vec<_>>()
                        .len()
                        == required_topics_set.len()
                })
                .collect();
        }

        Ok(res)
    }

    pub fn edit(
        &self,
        old_name: String,
        new_name: Option<String>,
        author: Option<String>,
        url: Option<String>,
        topics: Option<Vec<String>>,
        add_topics: Option<Vec<String>>,
        clear_topics: bool,
        remove_topics: Option<Vec<String>>,
    ) -> Result<Entry> {
        if new_name.is_none()
            && author.is_none()
            && url.is_none()
            && topics.is_none()
            && add_topics.is_none()
            && !clear_topics
            && remove_topics.is_none()
        {
            return Err(anyhow::anyhow!(
                "You gotta edit something, boi. Nice edit, such wow, much rlist"
            ));
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

        let (entry_id, mut entry) = if updates.len() == 0 {
            DBEntry::get_by_name_without_topics(&self.conn, old_name)?
        } else {
            let q = format!(
                "
                UPDATE rlist
                SET {u}
                WHERE name = :old_name
                RETURNING *;
                ",
                u = updates.join(", ")
            );
            let mut stmt = self.conn.prepare(q)?;
            stmt.bind_iter(bindings)?;
            if let sqlite::State::Done = stmt.next()? {
                return Err(anyhow::anyhow!(
                    "Could not get entry with name: {}",
                    old_name.as_str()
                ));
            }

            read_sql_response!(stmt, entry_id => i64, name => String, url => String, added => String, author => String);
            let author = opt_from_sql(author);

            (
                entry_id,
                Entry::new(name, url, author, Vec::new(), Some(added)),
            )
        };

        if clear_topics || topics.is_some() {
            DBEntry::unlink_all_topics(&self.conn, entry_id)?;
        }

        // --topics has precedence over --add-topics, and if the first is set, then the second won't do anything
        // --topics removes all topics associated with the entry and creates the new ones, whilst --add-topics just appends some topics
        let topics_to_add = if topics.is_some() { topics } else { add_topics };

        if topics_to_add.is_some() {
            let t = topics_to_add.unwrap();
            let topic_ids = DBTopic::create_many(&self.conn, &t)?;
            DBEntry::associate_with_topics(&self.conn, entry_id, topic_ids)?;
        }

        if remove_topics.is_some() {
            DBEntry::unlink_topics_by_name(&self.conn, entry_id, remove_topics.unwrap())?;
        }

        let total_topics = DBTopic::get_related_to(&self.conn, entry_id)?
            .into_iter()
            .map(|(_i, e)| e)
            .collect();

        entry.set_topics(total_topics);

        Ok(entry)
    }

    pub fn remove_by_topics(&self, topics: Vec<String>) -> Result<Vec<Entry>> {
        let mut res = Vec::new();
        for topic in topics {
            let old_entries = self.remove_by_topic(topic)?;
            res.extend(old_entries);
        }
        Ok(res)
    }

    pub fn remove_by_topic(&self, topic: String) -> Result<Vec<Entry>> {
        let q = "SELECT topic_id FROM topics WHERE name = :topic;";
        let mut stmt = self.conn.prepare(q)?;
        stmt.bind((":topic", topic.as_str()))?;

        if let sqlite::State::Done = stmt.next()? {
            return Err(anyhow::anyhow!("Topic not in topics"));
        }

        let topic_id = stmt.read::<i64, _>("topic_id")?;

        let q = "
        DELETE FROM rlist 
        WHERE entry_id IN (
            SELECT entry_id 
            FROM rlist_has_topic 
            WHERE topic_id = :topic_id
        ) RETURNING *;";
        let mut stmt = self.conn.prepare(q)?;
        stmt.bind((":topic_id", topic_id))?;

        let mut res = Vec::new();
        while let sqlite::State::Row = stmt.next()? {
            read_sql_response!(stmt, name => String, url => String, added => String, author => String);
            let author = opt_from_sql(author);

            //? Returning stuff with some defaults cause this function is currently only used with pretty_print (short version)
            let e = Entry::new(name, url, author, Vec::new(), Some(added));
            res.push(e);
        }

        _ = DBTopic::delete_by_id(&self.conn, topic_id)?;

        Ok(res)
    }
}

