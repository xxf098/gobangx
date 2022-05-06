use database_tree::{Database, Table};
use crate::sql::DbMetadata;
use crate::database::meta::Header;

pub struct Updater {
    // status: HashMap<String, u8>, // schema.table: status; 1: updating, 2: updated
    db_metadata: DbMetadata,
}

impl Default for Updater {
    fn default() -> Self {
        Self { db_metadata: DbMetadata::default() }
    }
}

impl Updater {

    // if already exist,then return false 
    pub fn update(&mut self, database: &Database, table: &Table, headers: &Vec<Header>) -> bool {
        let mut fullname = table.name.clone();
        if let Some(schema) = &table.schema {
            fullname = format!("{}.{}", schema, fullname);
        }
        fullname = format!{"{}.{}", database.name, fullname};
        let cols = self.db_metadata.tables.get(&fullname);
        if cols.is_some() {
            return false
        }
        let cols = headers.iter().map(|h| h.name.clone()).collect::<Vec<_>>();
        self.db_metadata.tables.insert(fullname, cols);
        return true
    }

    pub fn db_metadata(&self) -> &DbMetadata {
        &self.db_metadata
    }
}

