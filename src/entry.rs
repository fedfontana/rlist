use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::f32::consts::E;
use std::hash::{Hash, Hasher};

use crate::read_sql_response;
use crate::topic::Topic;
use crate::utils::{COLORS, opt_from_sql, ToSQL};

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    name: String,
    url: String,
    author: Option<String>,
    topics: Vec<String>,
    added: String,
}

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

    pub fn set_topics(&mut self, topics: Vec<String>) {
        self.topics = topics;
    }

    pub fn pretty_print_long(&self) {
        let topics_row = if self.topics.len() > 0 {
            format!(
                "Topics: {}\n",
                self.topics
                    .iter()
                    .map(|t| Topic::pretty_print(t.as_ref()))
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
}
