use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};
use colored::{Colorize, ColoredString};
use std::f32::consts::E;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    name: String,
    url: String,
    author: Option<String>,
    topics: Vec<String>,
    added: Option<String>,
}

const COLORS: [(u8, u8, u8); 5] = [
    (200, 10, 20), 
    (125, 30, 20), 
    (10, 20, 55), 
    (130, 130, 10), 
    (10, 200, 120),
];


impl Entry {
    pub fn new(name: String, url: String, author: Option<String>, topics: Vec<String>) -> Self {
        Self {
            name,
            url,
            author,
            topics,
            added: None,
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
        where T: AsRef<str> + Hash + Colorize {
        let mut hasher =  DefaultHasher::new();
        topic.hash(&mut hasher);
        let c = COLORS[hasher.finish() as usize % COLORS.len()];
        topic.on_truecolor(c.0, c.1, c.2).to_string()
    }

    pub fn pretty_print(&self) {
        // TODO determine topic color based on hash of topic
        println!("{name}: {url}{maybe_author}\nTopics: {topics}\nAdded on {added}",
            name = self.name.bold().truecolor(255, 165, 0),
            url = self.url.bright_blue().underline(),
            maybe_author = self.author.as_ref().map(|v| format!(" by {}", v.green())).unwrap_or("".into()),
            topics = self.topics.iter().map(|t| Entry::pretty_print_topic(t.as_ref())).collect::<Vec<_>>().join(", "),
            added = "2023-01-07 22:10:04",
        );
    }
}