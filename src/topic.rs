use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use uuid::Uuid;

use crate::utils::COLORS;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Topic {
    #[serde(skip_serializing, default = "Uuid::now_v7")]
    id: Uuid,
    pub name: String,
}

impl Topic {
    pub(crate) fn new(name: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
        }
    }

    pub(crate) fn pretty_print(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let c = COLORS[hasher.finish() as usize % COLORS.len()];
        self.name.on_truecolor(c.0, c.1, c.2).to_string()
    }
}

impl Hash for Topic {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
    }
}
