pub mod mysql;
pub mod postgres;
pub mod sqlite;
pub mod mssql;
pub mod meta;

pub use mysql::MySqlPool;
pub use postgres::PostgresPool;
pub use sqlite::SqlitePool;
pub use mssql::MssqlPool;
pub use meta::{ColType, Header, Value, ColumnMeta, ColumnConstraint};

use std::collections::HashMap;
use async_trait::async_trait;
use database_tree::{Child, Database, Table};
use crate::config::DatabaseType;

pub const RECORDS_LIMIT_PER_PAGE: u8 = 200;
pub const MYSQL_KEYWORDS: [&str; 1] = ["int"];
pub const POSTGRES_KEYWORDS: [&str; 1] = ["group"];
const INDENT: &str = "    ";

#[async_trait]
pub trait Pool: Send + Sync {
    async fn execute(&self, query: &str) -> anyhow::Result<ExecuteResult>;
    async fn query(&self, query: &str) -> anyhow::Result<QueryResult>;
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

    async fn get_columns2(
        &self,
        _database: &Database,
        _table: &Table,
    ) -> anyhow::Result<Vec<ColumnMeta>> {
        anyhow::bail!("just for postgress")
    }
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

pub struct QueryResult {
    pub headers: Vec<Header>,
    pub rows: Vec<Vec<Value>>,
    pub updated_rows: u64,
}

pub trait TableRow: std::marker::Send {
    fn fields(&self) -> Vec<String>;
    fn columns(&self) -> Vec<String>;
}

impl DatabaseType {

    pub fn drop_table(&self, database: &Database, table: &Table) -> String {
        match self {
            DatabaseType::Postgres => format!("drop table {}.{}.{}", database.name, table.pg_schema(),table.name),
            DatabaseType::MySql => format!("drop table {}.{}", database.name, table.name),
            _ => format!("drop table {}", table.name),
        }
    }

    pub async fn show_schema(&self, pool: &Box<dyn Pool>, database: &Database, table: &Table) -> anyhow::Result<String> {
        match self {
            DatabaseType::MySql => {
                let sql = format!("show create table {}.{}", database.name, table.name);
                let result = pool.query(&sql).await?;
                return self.table_ddl(result)
            },
            DatabaseType::Sqlite => { 
                let sql = format!("select name, sql from sqlite_schema where name = '{}';", table.name);
                let result = pool.query(&sql).await?;
                return self.table_ddl(result)
            },
            DatabaseType::Postgres => {
                let columns = pool.get_columns2(database, table).await?;
                let pkey_cols = self.primary_key_columns(pool, database, table).await?;
                let indexes = self.postgres_index(pool, table).await?;
                let foreign_defs = self.postgres_foreign(pool, table).await?;
                let unique_constraints = self.postgres_unique(pool, table).await?;
                postgres_table_ddl(table, columns, pkey_cols, indexes, foreign_defs, unique_constraints).await
            },
            _ => unimplemented!(),
        }
    }

    async fn postgres_index(&self, pool: &Box<dyn Pool>, table: &Table) -> anyhow::Result<Vec<String>> {
        let sql = format!(r#"WITH
        unique_and_pk_constraints AS (
          SELECT con.conname AS name
          FROM   pg_constraint con
          JOIN   pg_namespace nsp ON nsp.oid = con.connamespace
          JOIN   pg_class cls ON cls.oid = con.conrelid
          WHERE  con.contype IN ('p', 'u')
          AND    nsp.nspname = '{schema}'
          AND    cls.relname = '{table}'
        )
      SELECT indexName, indexdef
      FROM   pg_indexes
      WHERE  schemaname = '{schema}'
      AND    tablename = '{table}'
      AND    indexName NOT IN (SELECT name FROM unique_and_pk_constraints)"#, schema=table.pg_schema(), table=table.name);
      let result = pool.query(&sql).await?;
      let indexes = result.rows.iter().map(|v| v[1].to_string()).collect::<Vec<_>>();
      Ok(indexes)
    }

    async fn postgres_foreign(&self, pool: &Box<dyn Pool>, table: &Table) -> anyhow::Result<Vec<String>> {
        let sql = format!(r#"SELECT
        tc.table_schema, tc.constraint_name, tc.table_name, kcu.column_name,
        ccu.table_schema AS foreign_table_schema,
        ccu.table_name AS foreign_table_name,
        ccu.column_name AS foreign_column_name,
        rc.update_rule AS foreign_update_rule,
        rc.delete_rule AS foreign_delete_rule
    FROM
        information_schema.table_constraints AS tc
        JOIN information_schema.key_column_usage AS kcu
            ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage AS ccu
            ON tc.constraint_name = ccu.constraint_name
        JOIN information_schema.referential_constraints AS rc
            ON tc.constraint_name = rc.constraint_name
    WHERE constraint_type = 'FOREIGN KEY' AND tc.table_schema='{schema}' AND tc.table_name='{table}'"#, schema=table.pg_schema(), table=table.name);
        let result = pool.execute(&sql).await?;
        match result {
            ExecuteResult::Read{ rows, .. } => {
              let indexes = rows.iter().map(|v| {
                    format!("ALTER TABLE ONLY {}.{} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{}({}) ON UPDATE {} ON DELETE {}",
                        v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8])
              }).collect::<Vec<_>>();
              Ok(indexes)
            },
            _ => Ok(vec![]),
        }
    }

    async fn postgres_unique(&self, pool: &Box<dyn Pool>, table: &Table) -> anyhow::Result<HashMap<String, String>> {
        let sql = format!(r#"SELECT con.conname, pg_get_constraintdef(con.oid)
            FROM   pg_constraint con
            JOIN   pg_namespace nsp ON nsp.oid = con.connamespace
            JOIN   pg_class cls ON cls.oid = con.conrelid
            WHERE  con.contype = 'u'
            AND    nsp.nspname = '{}'
            AND    cls.relname = '{}';"#, table.pg_schema(), table.name);
        let result = pool.execute(&sql).await?;
        match result {
            ExecuteResult::Read{ rows, .. } => {
                let mut hm = HashMap::new();
                let _indexes = rows.iter().map(|v| {
                    let val = format!("ALTER TABLE {} ADD CONSTRAINT {} {};", table.name, v[0], v[1]);
                    hm.insert(v[0].data.clone(), val)
                }).collect::<Vec<_>>();
                Ok(hm)
            },
            _ => Ok(HashMap::new()),
        }
    }

    fn table_ddl(&self, result: QueryResult) -> anyhow::Result<String> {
        match self {
            DatabaseType::MySql | DatabaseType::Sqlite => {
                let mut s = String::new();
                let rows = result.rows;
                if rows.len() > 0 && rows[0].len() > 1 {
                    s = rows[0][1].to_string();
                }  
                Ok(s)
            },
            _ => unreachable!(),
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
            let result = pool.query(&sql).await?;
            let cols = result.rows.into_iter().flat_map(|row| row.into_iter().next().map(|c| c.data)).collect();
            return Ok(cols)
            },
            DatabaseType::MySql => {
                let sql = format!("SHOW INDEX FROM {}.{}", database.name, table.name);
                let result = pool.query(&sql).await?;
                let headers = result.headers;
                let index = headers.iter().position(|h| h.name.to_lowercase() == "key_name").unwrap_or(headers.len());
                let cols = result.rows.into_iter().flat_map(|row| row.get(index).filter(|c| c.data == "PRIMARY").map(|_| row.get(index+2).map(|c| c.data.clone())).flatten()).collect();
                return Ok(cols)
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

    pub fn update_row_by_column(&self, database: &Database, table: &Table, pkey: &str, pval: &str, header: &Header, val: &Value) -> String {
        // TODO: NULL header type
        match self {
            DatabaseType::MySql => format!("UPDATE {}.{} SET {} = '{}' where {} = '{}'", database.name, table.name, header.name, val.data, pkey, pval),
            DatabaseType::Sqlite => format!("UPDATE {} SET {} = '{}' where {} = '{}'", table.name, header.name, val.data, pkey, pval),
            DatabaseType::Postgres => format!("UPDATE {}.{}.{} SET {} = '{}' where {} = '{}'", database.name, table.pg_schema(), header.name, val.data, table.name, pkey, pval),
            _ => unimplemented!(),
        }
    }


    // handle null | handle value type
    pub fn insert_rows(&self, database: &Database, table: &Table, headers: &Vec<Header>, rows: &Vec<Vec<Value>>) -> String {
        let header_str = self.insert_headers(headers);
        match self {
            DatabaseType::Postgres => {
                let mut sqls = vec![];
                for row in rows {
                    let row_str = convert_row_str(row, headers);
                    let sql = format!("INSERT INTO {}.{} ({}) VALUES ({})", table.schema.clone().unwrap_or_else(|| "public".to_string()), table.name, header_str, row_str);
                    sqls.push(sql)
                }
                sqls.join(";\n")
            }
            DatabaseType::MySql => {
                let mut sqls = vec![];
                for row in rows {
                    let row_str = convert_row_str(row, headers);                  
                    let sql = format!("INSERT INTO {}.{} ({}) VALUES ({})", database.name, table.name, header_str, row_str);
                    sqls.push(sql)
                }
                sqls.join(";\n")
            },
            DatabaseType::Sqlite => {
                let mut sqls = vec![];
                for row in rows {
                    let row_str = convert_row_str(row, headers);                  
                    let sql = format!("INSERT INTO {} ({}) VALUES ({})", table.name, header_str, row_str);
                    sqls.push(sql)
                }
                sqls.join(";\n")
            },
            _ => unimplemented!(),
        }
    }

    fn is_keywords(&self, w: &str) -> bool {
        match self {
            DatabaseType::MySql => {
                MYSQL_KEYWORDS.iter().find(|kw| **kw == w.to_lowercase()).is_some()
            },
            DatabaseType::Postgres => {
                POSTGRES_KEYWORDS.iter().find(|kw| **kw == w.to_lowercase()).is_some()
            },
            _ => false,
        }
    }

    fn insert_headers(&self, headers: &Vec<Header>) -> String {
        match self {
            DatabaseType::MySql => {
                headers.iter().map(|h| {
                    if self.is_keywords(&h.name) { format!("`{}`", h.to_string()) } else { h.to_string() }
                }).collect::<Vec<String>>().join(", ")
            },
            DatabaseType::Postgres => {
                headers.iter().map(|h| {
                    if self.is_keywords(&h.name) { format!("\"{}\"", h.to_string()) } else { h.to_string() }
                }).collect::<Vec<String>>().join(", ")
            },
            _ => headers.iter().map(|h| h.to_string()).collect::<Vec<String>>().join(", ")
        }
    }
}

// TODO: getIndexDefs getForeignDefs getPolicyDefs getTableCheckConstraints getUniqueConstraints
async fn postgres_table_ddl(table: &Table, columns: Vec<ColumnMeta>, pkey_cols: Vec<String>, index_defs: Vec<String>, foreign_defs: Vec<String>, unique_constraints: HashMap<String, String>) -> anyhow::Result<String> {
    let mut ddl = format!("CREATE TABLE {}.{} (", table.pg_schema(), table.name);
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            ddl = format!("{},", ddl);
        }
        ddl = format!("{}\n{}", ddl, INDENT);
        ddl = format!("{} \"{}\" {}", ddl, col.name, col.get_data_type());
        if col.length > 0 {
            ddl = format!("{}({})", ddl, col.length);
        }
        if !col.nullable {
            ddl = format!("{} NOT NULL", ddl);
        }
        if col.default.is_some() && col.default.as_ref().unwrap().len() > 0 && !col.is_auto_increment {
            ddl = format!("{} DEFAULT {}", ddl, col.default.as_ref().unwrap());
        }
        if col.identity_generation.is_some() && col.identity_generation.as_ref().unwrap().len() > 0{
            ddl = format!("{} GENERATED {} AS IDENTITY", ddl, col.identity_generation.as_ref().unwrap())
        }
        if col.check.is_some() {
            let check = col.check.as_ref().unwrap();
            ddl =  format!("{} CONSTRAINT {} {}", ddl, check.name, check.definition);
        }
    }
    if pkey_cols.len() > 0 {
        ddl = format!("{},\n{}", ddl, INDENT);
        let pkey = pkey_cols.join("\", \"");
        ddl = format!("{} PRIMARY KEY (\"{}\")", ddl, pkey);
    }
    ddl = format!("{}\n);\n", ddl);
    for index_def in index_defs {
        ddl = format!("{}{};\n", ddl, index_def)
    }
    for foreign_def in foreign_defs {
        ddl = format!("{}{};\n", ddl, foreign_def)
    }
    for (_, constraint_def) in unique_constraints {
        ddl = format!("{}{};\n", ddl, constraint_def)
    }
    Ok(ddl.trim_end().to_string())
}

fn convert_row_str (row: &Vec<Value>, headers: &Vec<Header>) -> String {
    let mut row_str = String::new();
    for (i, v) in row.iter().enumerate() {
        let s = if v.is_null { "NULL".to_string() } else {
            let mut s = format!("'{}'", v.data);
            if let Some(header) = headers.get(i) {
                if header.is_no_quote() { s = v.data.clone() };
            };
            s
        };
        row_str = if row_str.len() == 0 { s.to_string() } else { format!("{}, {}", row_str, s) };
    };
    row_str
}

#[macro_export]
macro_rules! get_or_null {
    ($value:expr) => {
        $value.map_or(Value::default(), |v| Value::new(v.to_string()))
    };
}
