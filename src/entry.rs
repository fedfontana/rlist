use chrono::{DateTime, Utc};
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::f32::consts::E;
use std::hash::{Hash, Hasher};
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    name: String,
    url: String,
    author: Option<String>,
    topics: Vec<String>,
    added: String,
}

const COLORS: [(u8, u8, u8); 20] = [
    (200, 10, 20),
    (125, 30, 20),
    (130, 130, 10),
    (10, 150, 120),
    (220, 165, 0),
    (207, 64, 207),
    (255, 117, 43),
    (38, 169, 173),
    (114, 39, 219),
    (219, 39, 78),
    (60, 105, 230),
    (60, 230, 130),
    (5, 171, 74),
    (105, 201, 14),
    (15, 103, 135),
    (161, 66, 51),
    (120, 89, 6),
    (245, 44, 44),
    (230, 195, 20),
    (5, 2, 207),
];

impl Entry {
    pub fn new(
        name: String,
        url: String,
        author: Option<String>,
        topics: Vec<String>,
        added: Option<String>,
    ) -> Self {
        Self {
            name,
            url,
            author,
            topics,
            added: added.unwrap_or_default(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    pub fn topics(&self) -> &Vec<String> {
        &self.topics
    }

    pub fn add_topic(&mut self, topic: String) {
        self.topics.push(topic)
    }

    fn pretty_print_topic<T>(topic: T) -> String
    where
        T: AsRef<str> + Hash + Colorize,
    {
        let mut hasher = DefaultHasher::new();
        topic.hash(&mut hasher);
        let c = COLORS[hasher.finish() as usize % COLORS.len()];
        topic.on_truecolor(c.0, c.1, c.2).to_string()
    }

    pub fn pretty_print_long(&self) {
        let topics_row = if self.topics.len() > 0 {
            format!(
                "Topics: {}\n",
                self.topics
                    .iter()
                    .map(|t| Entry::pretty_print_topic(t.as_ref()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            "".to_string()
        };

        println!(
            "{name}: {url}{maybe_author}\n{topics_row}Added on {added}",
            name = self.name.bold().truecolor(255, 165, 0),
            url = self.url.bright_blue().underline(),
            maybe_author = self
                .author
                .as_ref()
                .map(|v| format!(" by {}", v.green()))
                .unwrap_or("".into()),
            added = self.added,
        );
    }

    pub fn pretty_print(&self) {
        println!(
            "{name}: {url}{maybe_author}",
            name = self.name.bold().truecolor(255, 165, 0),
            url = self.url.bright_blue().underline(),
            maybe_author = self
                .author
                .as_ref()
                .map(|v| format!(" by {}", v.green()))
                .unwrap_or("".into()),
        );
    }

    pub(crate) fn associate_with_topics(conn: &sqlite::Connection, entry_id: i64, topic_ids: Vec<i64>) -> Result<()> {
        let q = format!(
            "INSERT INTO rlist_has_topic (entry_id, topic_id) VALUES {};",
            topic_ids.iter().map(|_t| "(?, ?)").collect::<Vec<_>>().join(", ")
        );

        let mut stmt = conn.prepare(q)?;
        let bindings = (0..2*topic_ids.len()).map(|idx| {
            if idx % 2 == 0 {
                (idx+1, entry_id)
            } else {
                (idx+1, topic_ids[(idx-1)/2])
            }
        }).collect::<Vec<_>>();
        
        stmt.bind(bindings.as_slice())?;
        stmt.next()?;

        Ok(())
    }

    pub(crate) fn create(conn: &sqlite::Connection, name: &str, url: &str, author: Option<&str>) -> Result<i64> {
        let q= "INSERT INTO rlist (name, url, author) VALUES (:name, :url, :author) RETURNING entry_id";
        let mut stmt = conn.prepare(q)?;
        let a = if let Some(a) = author {
            a.as_ref()
        } else {
            "NULL"
        };

        stmt.bind(
            &[
                (":name", name),
                (":url", url),
                (":author", a),
            ][..],
        )?;

        if let sqlite::State::Row = stmt.next()? {
            let entry_id = stmt.read::<i64, _>("entry_id")?;
            return Ok(entry_id);
        }

        Err(anyhow::anyhow!("Could not insert entry with name: {name}"))
    }
}
