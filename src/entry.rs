use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Entry {
    url: String,
    author: Option<String>,
    topics: Vec<String>,
    date: DateTime<Utc>
}

impl Entry {
    pub fn new(url: String, author: Option<String>, topics: Vec<String>, date: DateTime<Utc>) -> Self {
        Self {
            url,
            author,
            topics,
            date,
        }
    }
}