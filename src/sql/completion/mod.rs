pub mod plain;
pub mod advance;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use crate::config::{DatabaseType};
pub use advance::Updater;

pub struct DbMetadata {
    columns: HashMap<(String, String), Vec<String>>, // {"(schema|database, table)": vec!["col1", "col2", "col3"]}
    views: HashMap<(String, String), Vec<String>>,
    current_db: String,
    databases: Vec<String>,
    schemas: Vec<String>, // schema
    tables: Vec<String>,
}

impl Default for DbMetadata {
    fn default() -> Self {
        Self { 
            columns: HashMap::new(), 
            views: HashMap::new(),
            current_db: "".to_string(), 
            databases: vec![], 
            schemas: vec![],
            tables: vec![], 
        }
    }
}

pub trait Completion {
    fn new(db_type: DatabaseType, candidates: Vec<String>) ->Self;
    fn complete(&self, full_text: &str) -> Vec<String>;
    fn update(&mut self, candidates: &[String], db_metadata: Option<Arc<RwLock<DbMetadata>>>);
}

