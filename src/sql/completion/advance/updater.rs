use std::rc::Rc;
use std::cell::RefCell;
use database_tree::{Database, Table};
use crate::sql::DbMetadata;
use crate::database::meta::Header;

pub struct Updater {
    // status: HashMap<String, u8>, // schema.table: status; 1: updating, 2: updated
    db_metadata: Rc<RefCell<DbMetadata>>,
}

impl Default for Updater {
    fn default() -> Self {
        Self { db_metadata: Rc::new(RefCell::new(DbMetadata::default())) }
    }
}

impl Updater {

    // if already exist,then return false 
    pub fn update(&mut self, database: &Database, table: &Table, headers: &Vec<Header>) -> bool {
        let key = (table.schema.clone().unwrap_or("".to_string()),table.name.clone());
        let db_metadata = self.db_metadata.borrow();
        let cols = db_metadata.tables.get(&key);
        if cols.is_some() {
            return false
        }
        std::mem::drop(db_metadata);
        let cols = headers.iter().map(|h| h.name.clone()).collect::<Vec<_>>();
        let mut db_metadata = self.db_metadata.borrow_mut();
        db_metadata.tables.insert(key, cols);
        db_metadata.dbname = database.name.clone();
        return true
    }

    pub fn db_metadata(&self) -> Rc<RefCell<DbMetadata>> {
        self.db_metadata.clone()
    }
}

