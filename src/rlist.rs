use crate::entry::Entry;

pub struct RList {
    pub content: Vec<Entry>,
}

impl RList {
    pub fn new(content: Vec<Entry>) -> Self {
        Self {
            content
        }
    }

    pub fn add(&mut self, new_entry: Entry) -> bool {
        if self.content.iter().position(|e| e.id() == new_entry.id()).is_some() {
            return false;
        }
        self.content.push(new_entry);
        true
    }

    pub fn remove_with_id(&mut self, id: impl AsRef<str>) -> Option<Entry> {
        if let Some(idx) = self.content.iter().position(|e| e.id() == id.as_ref()) {
            Some(self.content.remove(idx))
        } else {
            None
        }
    }
    
}