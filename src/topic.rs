use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use anyhow::Result;
use colored::Colorize;
use std::hash::Hash;

use crate::{utils::COLORS, read_sql_response};

pub(crate) struct Topic {}

impl Topic {
    pub(crate) fn pretty_print<T>(topic: T) -> String
    where
        T: AsRef<str> + Hash + Colorize,
    {
        let mut hasher = DefaultHasher::new();
        topic.hash(&mut hasher);
        let c = COLORS[hasher.finish() as usize % COLORS.len()];
        topic.on_truecolor(c.0, c.1, c.2).to_string()
    }
}
