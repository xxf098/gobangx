pub mod mysql;
pub mod postgres;
pub mod sqlite;
pub mod mssql;

pub use mysql::MySqlPool;
pub use postgres::PostgresPool;
pub use sqlite::SqlitePool;
pub use mssql::MssqlPool;

use async_trait::async_trait;
use database_tree::{Child, Database, Table};
use crate::config::DatabaseType;

pub const RECORDS_LIMIT_PER_PAGE: u8 = 200;

#[async_trait]
pub trait Pool: Send + Sync {
    async fn execute(&self, query: &String) -> anyhow::Result<ExecuteResult>;
    async fn get_databases(&self) -> anyhow::Result<Vec<Database>>;
    async fn get_tables(&self, database: String) -> anyhow::Result<Vec<Child>>;
    async fn get_records(
        &self,
        database: &Database,
        table: &Table,
        page: u16,
        filter: Option<String>,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)>;
    async fn get_columns(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>>;
    async fn get_constraints(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>>;
    async fn get_foreign_keys(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>>;
    async fn get_indexes(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>>;
    async fn close(&self);

    fn database_type(&self) -> DatabaseType;
}

pub enum ExecuteResult {
    Read {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        database: Database,
        table: Table,
    },
    Write {
        updated_rows: u64,
    },
}

pub trait TableRow: std::marker::Send {
    fn fields(&self) -> Vec<String>;
    fn columns(&self) -> Vec<String>;
}

impl DatabaseType {

    pub fn drop_table(&self, database: &Database, table: &Table) -> String {
        match self {
            DatabaseType::Postgres => format!("drop table {}.{}.{}", database.name, table.schema.clone().unwrap_or_else(|| "public".to_string()),table.name),
            DatabaseType::MySql => format!("drop table {}.{}", database.name, table.name),
            _ => format!("drop table {}", table.name),
        }
    }

    pub fn show_schema(&self, database: &Database, table: &Table) -> String {
        match self {
            DatabaseType::MySql => format!("show create table {}.{}", database.name, table.name),
            _ => format!(""),
        }
    }
}

#[macro_export]
macro_rules! get_or_null {
    ($value:expr) => {
        $value.map_or("NULL".to_string(), |v| v.to_string())
    };
}
