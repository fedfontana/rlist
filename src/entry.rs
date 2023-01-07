use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    name: String,
    url: String,
    author: Option<String>,
    topics: Vec<String>,
}

impl Entry {
    pub fn new(name: String, url: String, author: Option<String>, topics: Vec<String>) -> Self {
        Self {
            name,
            url,
            author,
            topics,
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
}