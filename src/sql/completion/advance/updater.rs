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
    pub fn update_columns(&mut self, database: &Database, table: &Table, headers: &Vec<Header>) -> bool {
        let key = (table.schema.clone().unwrap_or(database.name.clone()),table.name.clone());
        let db_metadata = self.db_metadata.read().unwrap();
        let cols = db_metadata.columns.get(&key);
        if cols.is_some() {
            return false
        }
        std::mem::drop(db_metadata);
        let cols = headers.iter().map(|h| h.name.clone()).collect::<Vec<_>>();
        let mut db_metadata = self.db_metadata.write().unwrap();
        db_metadata.columns.insert(key, cols);
        db_metadata.current_db = database.name.clone();
        return true
    }

    pub fn update_databases(&mut self, databases: Vec<String>) {
        let mut db_metadata = self.db_metadata.write().unwrap();
        databases.into_iter().for_each(|s| {
            if db_metadata.databases.iter().find(|s1| *s1 == &s).is_none() {
                db_metadata.databases.push(s)
            }
        });
    }

    pub fn update_schemas(&mut self, schemas: Vec<String>) {
        let mut db_metadata = self.db_metadata.write().unwrap();
        schemas.into_iter().for_each(|s| {
            if db_metadata.schemas.iter().find(|s1| *s1 == &s).is_none() {
                db_metadata.schemas.push(s)
            }
        });
    }

    pub fn update_tables(&mut self, tables: Vec<String>) {
        let mut db_metadata = self.db_metadata.write().unwrap();
        tables.into_iter().for_each(|s| {
            if db_metadata.tables.iter().find(|s1| *s1 == &s).is_none() {
                db_metadata.tables.push(s)
            }
        });
    }

    pub fn db_metadata(&self) -> Arc<RwLock<DbMetadata>> {
        self.db_metadata.clone()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_schemas() {
        let mut u = Updater::default();
        let schemas = vec!["schema1".to_string(), "schema2".to_string(), "schema3".to_string()];
        u.update_schemas(schemas);
        let db_metadata = u.db_metadata();
        let db_metadata = db_metadata.read().unwrap();
        assert_eq!(db_metadata.schemas, vec!["schema1", "schema2", "schema3"])
    }

    #[test]
    fn test_update_tables() {
        let mut u = Updater::default();
        let tables = vec!["table1".to_string(), "table2".to_string(), "table3".to_string()];
        u.update_tables(tables);
        let db_metadata = u.db_metadata();
        let db_metadata = db_metadata.read().unwrap();
        assert_eq!(db_metadata.tables, vec!["table1", "table2", "table3"])
    }
}
