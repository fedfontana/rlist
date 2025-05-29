use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::topic::Topic;

// TODO: make all the ids be UUIDs and generate them at creation time. Also enable the uuid feature
// on rusqlite. Skip serializing them. Add them to the model struct
// TODO: impl custom Deserialize that generates a random UUID when importing stuff. When importing,
// make sure that we do not create duplicaed entries, and just merge them/signal the conflict. If
// everything is the same, just add the topics to the old one
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    #[serde(skip_serializing, default = "Uuid::now_v7")]
    id: Uuid,
    pub name: String,
    pub url: String,
    pub author: Option<String>,
    topics: Vec<Topic>,
    pub added: jiff::Zoned,
}

impl Entry {
    pub fn new(name: String, url: String, author: Option<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            url,
            author,
            topics: Vec::new(),
            added: jiff::Zoned::now().round(jiff::Unit::Second).unwrap(),
        }
    }

    pub fn add_topic(&mut self, topic: &str) {
        self.topics.push(Topic::new(topic.to_string()))
    }

    pub fn topics<'t>(&'t self) -> impl Iterator<Item = &'t str> {
        self.topics.iter().map(|t| t.name.as_ref())
    }

    /// Prints the entry to stdout.
    /// If `!long`, then it will only print `name: url [by author]`
    /// otherwise, it will also print the topics and `self.added`
    pub fn pretty_print(&self, long: bool, fmt_str: impl AsRef<str>) -> Result<()> {
        let topics_row = if long && self.topics.len() > 0 {
            format!(
                "\nTopics: {}",
                self.topics
                    .iter()
                    .map(|t| t.pretty_print())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        let added_row = if long {
            format!(
                "\nAdded on {}",
                // TODO: return an error/setup a default formatting to use in case the provided one
                // fails. If the user-chosen format fails, then also print an error to stderr
                jiff::fmt::strtime::format(fmt_str.as_ref(), &self.added).unwrap()
            )
        } else {
            String::new()
        };

        // TODO:
        // const ORANGE: colored::Color = (255u8, 165u8, 0u8).into();

        println!(
            "{name}: {url}{maybe_author}{topics_row}{added_row}",
            name = self.name.bold().truecolor(255, 165, 0), // orange
            url = self.url.bright_blue().underline(),
            maybe_author = self
                .author
                .as_ref()
                .map(|v| format!(" by {}", v.green()))
                .unwrap_or("".into()),
        );

        Ok(())
    }
}
