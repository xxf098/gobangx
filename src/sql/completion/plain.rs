use super::{Completion, DbMetadata};
use super::advance::last_word;
use crate::config::{DatabaseType};

pub struct Plain {
    candidates: Vec<String>,
}

impl Plain {
}


impl Completion for Plain {

    fn new(_db_type: DatabaseType,mut candidates: Vec<String>) -> Self {
        // let mut candidates: Vec<_> = candidates.iter().map(|w| w.to_string()).collect();
        candidates.sort();
        candidates.dedup();
        Self { candidates }
    }

    fn complete(&self, full_text: &str) -> Vec<String> {
        let word = last_word(full_text, "most_punctuations");
        self.candidates.iter().filter(move |c| {
            (c.starts_with(word.to_lowercase().as_str())
                || c.starts_with(word.to_uppercase().as_str()))
                && !word.is_empty()
        }).map(|c| c.clone()).collect::<Vec<_>>()
    }

    fn update(&mut self, candidates: &[String], db_metadata: DbMetadata) {
        for candidate in candidates {
            if self.candidates.iter().find(|x| *x == candidate).is_none() {
                self.candidates.push(candidate.clone())
            }
        }

        for (k, cols) in db_metadata.tables {
            for col in cols {
                if self.candidates.iter().find(|x| **x == col).is_none() {
                    self.candidates.push(col.clone())
                }
            }
        }
    }
}

