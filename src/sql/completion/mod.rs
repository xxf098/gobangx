pub mod plain;
pub mod advance;

use crate::config::{DatabaseType};

pub trait Completion {
    fn new(db_type: DatabaseType, candidates: Vec<String>) ->Self;
    fn complete(&self, full_text: String, word: &String) -> Vec<&String>;
    fn update_candidates(&mut self, candidates: &[String]);
}

