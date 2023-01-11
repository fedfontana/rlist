use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::topic::Topic;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    pub url: String,
    pub author: Option<String>,
    pub topics: Vec<String>,
    pub added: String,
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

    pub fn pretty_print(&self, long: bool) {
        let topics_row = if long && self.topics.len() > 0 {
            format!(
                "\nTopics: {}",
                self.topics
                    .iter()
                    .map(|t| Topic::pretty_print(t.as_ref()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        let added_row = if long {
            format!("\nAdded on {}", self.added)
        } else {
            String::new()
        };

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
    }
}
