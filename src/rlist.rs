use crate::entry::Entry;
use anyhow::Result;
use dateparser::DateTimeUtc;
use std::path::PathBuf;
use std::{collections::HashSet, path::Path, str::FromStr};

use crate::db::{entry::DBEntry, topic::DBTopic};
use crate::read_sql_response;
use crate::utils::{dt_to_string, opt_from_sql};

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
            other => Err(anyhow::anyhow!("Option \"{other}\" not recognized")),
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
    /// Creates the db file, initializes the tables and establishes a connection to the sqlite db
    /// Forwards the errors raised by the called functions, such as std::fs and sqlite ones.
    pub fn init(db_file_path: Option<PathBuf>) -> Result<Self> {
        let p = if db_file_path.is_none() {
            let home_dir_path =
                dirs::home_dir().ok_or(anyhow::anyhow!("Could not find home folder"))?;
            let rlist_dir = Path::new(home_dir_path.as_os_str()).join("rlist");
            std::fs::create_dir_all(&rlist_dir)?;

            rlist_dir.join("rlist.sqlite")
        } else {
            let pb = db_file_path.unwrap();
            let rlist_dir = Path::new(pb.as_os_str());
            std::fs::create_dir_all(&rlist_dir.parent().ok_or(anyhow::anyhow!(
                "Could not create directories needed to create the reading list"
            ))?)?;

            rlist_dir.to_path_buf()
        };

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

    /// Adds the entry to the database. Returns Ok(()) if the entry was added
    pub fn add(
        &self,
        name: String,
        url: String,
        author: Option<String>,
        topics: Vec<String>,
    ) -> Result<Entry> {
        let (entry_id, mut entry) =
            DBEntry::create(&self.conn, name.as_str(), url.as_str(), author.as_deref())?;

        if topics.len() > 0 {
            let topic_ids = DBTopic::create_many(&self.conn, &topics)?;
            DBEntry::associate_with_topics(&self.conn, entry_id, topic_ids)?;
        }
        entry.topics = topics;

        Ok(entry)
    }

    /// Removes the entry by name. Returns Ok(the old entry if it existed)
    pub fn remove_by_name(&self, name: String) -> Result<Entry> {
        DBEntry::remove_by_name(&self.conn, name.clone())
    }

    /// Returns the list of entries that match the query.
    /// If query is set, then it will be contained in each of the entries' names
    /// If author is set, then only entries with an author that contains this value will be returned
    /// Same with url
    /// If topics is set, then the returned enties will be contained in __all__ of those topics. If `or` is set to true,
    /// then the function will return the entries that are in __at least one__ of the topics.
    /// `from` and `to` control the range of the dates in which the returned entries were created.
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
        or: bool,
    ) -> Result<Vec<Entry>> {
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

            // If the entry is already in the vector, then just add the current topic to the entry's topics
            if let Some(pos) = res.iter().position(|e| e.name == name) {
                if topic.is_some() {
                    res[pos].topics.push(topic.unwrap());
                }
            } else {
                // else create a new entry
                read_sql_response!(stmt, url => String, added => String, author => String);
                let author = opt_from_sql(author);

                let topics = topic.map(|t| vec![t]).unwrap_or_default();

                let entry = Entry::new(name.clone(), url, author, topics, Some(added));
                res.push(entry);
            }
        }

        // Filter out the topics based on topics
        if let Some(topics) = topics {
            let required_topics_set = topics.iter().collect::<HashSet<_>>();

            res = res
                .into_iter()
                .filter(|entry| {
                    let entry_topics_set = entry.topics.iter().collect::<HashSet<_>>();

                    let intersection_len = entry_topics_set
                        .intersection(&required_topics_set)
                        .collect::<Vec<_>>()
                        .len();

                    if or {
                        intersection_len > 0
                    } else {
                        intersection_len == required_topics_set.len()
                    }
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
        // If no edit is set, then return an error
        if new_name.is_none()
            && author.is_none()
            && url.is_none()
            && topics.is_none()
            && add_topics.is_none()
            && !clear_topics
            && remove_topics.is_none()
        {
            return Err(anyhow::anyhow!("No edit options were given"));
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

        // If there are no updates on the entry to be made, then just get the entry and its id.
        let (entry_id, mut entry) = if updates.len() == 0 {
            DBEntry::get_by_name_without_topics(&self.conn, old_name)?
        } else {
            // else perform the updates and construct a new Entry with the resulting data
            let q = format!(
                "UPDATE rlist
                SET {u}
                WHERE name = :old_name
                RETURNING *;",
                u = updates.join(", ")
            );
            let mut stmt = self.conn.prepare(q)?;
            stmt.bind_iter(bindings)?;
            if let sqlite::State::Done = stmt.next()? {
                return Err(anyhow::anyhow!(
                    "Could not find any entry in your reading list with name {}",
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

        entry.topics = DBTopic::get_related_to(&self.conn, entry_id)?
            .into_iter()
            .map(|(_i, e)| e)
            .collect();

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

    /// Removes all of the entries that are in `topic` and returns them
    pub fn remove_by_topic(&self, topic: String) -> Result<Vec<Entry>> {
        let topic_id = DBTopic::get_id_from_name(&self.conn, topic.as_str())?;

        let entries = self.query(
            None,
            Some(vec![topic]),
            None,
            None,
            None,
            false,
            None,
            None,
            false,
        )?;

        DBEntry::remove_related_to(&self.conn, topic_id)?;

        Ok(entries)
    }

    pub(crate) fn dump_all(&self) -> Result<Vec<Entry>> {
        DBEntry::get_all_complete(&self.conn)
    }

    /// Creates all of the entries provided.
    pub(crate) fn import(&self, entries: Vec<Entry>) -> Result<u64> {
        let mut c = 0;
        for e in entries {
            if let Ok((entry_id, _entry)) = DBEntry::create(
                &self.conn,
                e.name.as_str(),
                e.url.as_str(),
                e.author.as_deref(),
            ) {
                if let Ok(topic_ids) = DBTopic::create_many(&self.conn, &e.topics) {
                    if DBEntry::associate_with_topics(&self.conn, entry_id, topic_ids).is_ok() {
                        c += 1;
                    }
                }
            }
        }
        Ok(c)
    }
}
