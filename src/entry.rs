use colored::Colorize;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

use crate::{topic::Topic, utils::sql_string_to_dt};

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

    /// Prints the entry to stdout.
    /// If `!long`, then it will only print `name: url [by author]`
    /// otherwise, it will also print the topics and `self.added`
    pub fn pretty_print(&self, long: bool, fmt_str: impl AsRef<str>) -> Result<()> {
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
            let dt = sql_string_to_dt(self.added.as_str()).context("Could not format datetime in the desired format")?;

            format!("\nAdded on {}", dt.format(fmt_str.as_ref()))
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

        Ok(())
    }
}
