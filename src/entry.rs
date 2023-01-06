use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    url: String,
    author: Option<String>,
    topics: Vec<String>,
    date: DateTime<Utc>,
    id: String,
}

impl Entry {
    pub fn new(id: String, url: String, author: Option<String>, topics: Vec<String>, date: DateTime<Utc>) -> Self {
        Self {
            url,
            author,
            topics,
            date,
            id
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn url(&self) -> &str { 
        self.url.as_str()
    }

    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    pub fn date(&self) -> &DateTime<Utc> {
        &self.date
    }

    pub fn topics(&self) -> &Vec<String> {
        self.topics.as_ref()
    }
}