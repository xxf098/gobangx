pub mod plain;
pub mod advance;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use crate::config::{DatabaseType};
pub use advance::Updater;

pub struct DbMetadata {
    tables: HashMap<(String, String), Vec<String>>, // {"(schema, table)": vec!["col1", "col2", "col3"]}
    views: HashMap<(String, String), Vec<String>>,
    dbname: String
}

impl Default for DbMetadata {
    fn default() -> Self {
        Self { tables: HashMap::new(), views: HashMap::new(), dbname: "".to_string() }
    }
}

pub trait Completion {
    fn new(db_type: DatabaseType, candidates: Vec<String>) ->Self;
    fn complete(&self, full_text: &str) -> Vec<String>;
    fn update(&mut self, candidates: &[String], db_metadata: Option<Arc<RwLock<DbMetadata>>>);
}

