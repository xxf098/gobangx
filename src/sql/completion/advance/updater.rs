use std::sync::{Arc, RwLock};
use database_tree::{Database, Table};
use crate::sql::DbMetadata;
use crate::database::meta::Header;

pub struct Updater {
    // status: HashMap<String, u8>, // schema.table: status; 1: updating, 2: updated
    db_metadata: Arc<RwLock<DbMetadata>>,
}

impl Default for Updater {
    fn default() -> Self {
        Self { db_metadata: Arc::new(RwLock::new(DbMetadata::default())) }
    }
}

impl Updater {

    // if already exist,then return false 
    pub fn update(&mut self, database: &Database, table: &Table, headers: &Vec<Header>) -> bool {
        let key = (table.schema.clone().unwrap_or(database.name.clone()),table.name.clone());
        let db_metadata = self.db_metadata.read().unwrap();
        let cols = db_metadata.tables.get(&key);
        if cols.is_some() {
            return false
        }
        std::mem::drop(db_metadata);
        let cols = headers.iter().map(|h| h.name.clone()).collect::<Vec<_>>();
        let mut db_metadata = self.db_metadata.write().unwrap();
        db_metadata.tables.insert(key, cols);
        db_metadata.dbname = database.name.clone();
        return true
    }

    pub fn db_metadata(&self) -> Arc<RwLock<DbMetadata>> {
        self.db_metadata.clone()
    }
}

