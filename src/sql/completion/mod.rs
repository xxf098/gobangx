pub mod plain;
pub mod advance;

use std::collections::HashMap;
use crate::config::{DatabaseType};
pub use advance::Updater;

pub struct DbMetadata {
    tables: HashMap<String, Vec<String>>, // {"schema.table": vec!["col1", "col2", "col3"]}
    dbname: String
}

impl Default for DbMetadata {
    fn default() -> Self {
        Self { tables: HashMap::new(), dbname: "".to_string() }
    }
}

pub trait Completion {
    fn new(db_type: DatabaseType, candidates: Vec<String>) ->Self;
    fn complete(&self, full_text: &str) -> Vec<String>;
    fn update(&mut self, candidates: &[String], db_metadata: Option<&DbMetadata>);
}

