pub mod mysql;
pub mod postgres;
pub mod sqlite;
pub mod mssql;
pub mod meta;

pub use mysql::MySqlPool;
pub use postgres::PostgresPool;
pub use sqlite::SqlitePool;
pub use mssql::MssqlPool;
pub use meta::{ColType, Header, Value};

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
    ) -> anyhow::Result<(Vec<Header>, Vec<Vec<Value>>)>;
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
        headers: Vec<Header>,
        rows: Vec<Vec<Value>>,
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
            DatabaseType::Sqlite => format!("select name, sql from sqlite_schema where name = '{}';", table.name),
            _ => unimplemented!(),
        }
    }

    pub async fn primary_key_columns(&self, pool: &Box<dyn Pool>, database: &Database, table: &Table) -> anyhow::Result<Vec<String>> {
        let columns = vec![];
        match self {
            DatabaseType::Postgres => {
                let sql = format!(r#"SELECT
                kcu.column_name,tc.table_schema, tc.constraint_name, tc.table_name
            FROM
                information_schema.table_constraints AS tc
                JOIN information_schema.key_column_usage AS kcu
                    USING (table_schema, table_name, constraint_name)
            WHERE constraint_type = 'PRIMARY KEY' AND tc.table_schema='{}' AND tc.table_name='{}' ORDER BY kcu.ordinal_position"#, 
                table.schema.clone().unwrap_or_else(|| "public".to_string()), table.name);
            let result = pool.execute(&sql).await?;
            match result {
                ExecuteResult::Read{ rows, .. } => {
                    let cols = rows.into_iter().flat_map(|row| row.into_iter().next().map(|c| c.data)).collect();
                    return Ok(cols)
                },
                _ => {}
            };
            },
            DatabaseType::MySql => {
                let sql = format!("SHOW INDEX FROM {}.{}", database.name, table.name);
                let result = pool.execute(&sql).await?;
                match result {
                    ExecuteResult::Read{ headers, rows, .. } => {
                        let index = headers.iter().position(|h| h.name.to_lowercase() == "key_name").unwrap_or(headers.len());
                        let cols = rows.into_iter().flat_map(|row| row.get(index).filter(|c| c.data == "PRIMARY").map(|_| row.get(index+2).map(|c| c.data.clone())).flatten()).collect();
                        return Ok(cols)
                    },
                    _ => {}
                };

            },
            _ => {},
        };
        Ok(columns)
    }

    pub fn delete_row_by_column(&self, database: &Database, table: &Table, col: &str, val: &str) -> String {
        match self {
            DatabaseType::MySql => format!("delete from {}.{} where {} = '{}'", database.name, table.name, col, val),
            DatabaseType::Sqlite => format!("delete from {} where {} = '{}'", table.name, col, val),
            DatabaseType::Postgres => format!("delete from {}.{}.{} where {} = '{}'", database.name, table.schema.clone().unwrap_or_else(|| "public".to_string()), table.name, col, val),
            _ => unimplemented!(),
        }
    }

    // handle null | handle value type
    pub fn insert_rows(&self, database: &Database, table: &Table, headers: &Vec<Header>, rows: &Vec<Vec<Value>>) -> String {
        let header_str = headers.iter().map(|h| h.to_string()).collect::<Vec<String>>().join(", ");
        match self {
            DatabaseType::Postgres => {
                let mut sqls = vec![];
                for row in rows {
                    let mut row_str = String::new();
                    for v in row {
                        let s = if v.is_null { "NULL".to_string() } else { format!("'{}'", v.data) };
                        row_str = if row_str.len() == 0 { s.to_string() } else { format!("{}, {}", row_str, s) };
                    }
                    let sql = format!("INSERT INTO {}.{} ({}) VALUES ({})", table.schema.clone().unwrap_or_else(|| "public".to_string()), table.name, header_str, row_str);
                    sqls.push(sql)
                }
                sqls.join(";")
            }
            DatabaseType::MySql => {
                let mut sqls = vec![];
                for row in rows {
                    let mut row_str = String::new();
                    for v in row {
                        let s = if v.is_null { "NULL".to_string() } else { format!("'{}'", v.data) };
                        row_str = if row_str.len() == 0 { s.to_string() } else { format!("{}, {}", row_str, s) };
                    }                    let sql = format!("INSERT INTO {}.{} ({}) VALUES ({})", database.name, table.name, header_str, row_str);
                    sqls.push(sql)
                }
                sqls.join(";")
            },
            _ => unimplemented!(),
        }
    }
}

#[macro_export]
macro_rules! get_or_null {
    ($value:expr) => {
        $value.map_or(Value::default(), |v| Value::new(v.to_string()))
    };
}
