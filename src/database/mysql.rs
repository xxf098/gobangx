use crate::get_or_null;
use crate::config::DatabaseType;
use super::{ExecuteResult, QueryResult, Pool, TableRow, RECORDS_LIMIT_PER_PAGE, Header, ColType, Value};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use database_tree::{Child, Database, Table};
use futures::TryStreamExt;
use sqlx::mysql::{MySqlColumn, MySqlPoolOptions, MySqlRow, MySql};
use sqlx::{Column as _, Row as _, TypeInfo as _};
use sqlx::decode::Decode;
use std::time::Duration;

pub struct MySqlPool {
    pool: sqlx::mysql::MySqlPool,
}

impl MySqlPool {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool: MySqlPoolOptions::new()
                .connect_timeout(Duration::from_secs(5))
                .connect(database_url)
                .await?,
        })
    }
}

pub struct Constraint {
    name: String,
    column_name: String,
}

impl TableRow for Constraint {
    fn fields(&self) -> Vec<String> {
        vec!["name".to_string(), "column_name".to_string()]
    }

    fn columns(&self) -> Vec<String> {
        vec![self.name.to_string(), self.column_name.to_string()]
    }
}

pub struct Column {
    name: Option<String>,
    r#type: Option<String>,
    null: Option<String>,
    default: Option<String>,
    comment: Option<String>,
}

impl TableRow for Column {
    fn fields(&self) -> Vec<String> {
        vec![
            "name".to_string(),
            "type".to_string(),
            "null".to_string(),
            "default".to_string(),
            "comment".to_string(),
        ]
    }

    fn columns(&self) -> Vec<String> {
        vec![
            self.name
                .as_ref()
                .map_or(String::new(), |name| name.to_string()),
            self.r#type
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
            self.null
                .as_ref()
                .map_or(String::new(), |null| null.to_string()),
            self.default
                .as_ref()
                .map_or(String::new(), |default| default.to_string()),
            self.comment
                .as_ref()
                .map_or(String::new(), |comment| comment.to_string()),
        ]
    }
}

pub struct ForeignKey {
    name: Option<String>,
    column_name: Option<String>,
    ref_table: Option<String>,
    ref_column: Option<String>,
}

impl TableRow for ForeignKey {
    fn fields(&self) -> Vec<String> {
        vec![
            "name".to_string(),
            "column_name".to_string(),
            "ref_table".to_string(),
            "ref_column".to_string(),
        ]
    }

    fn columns(&self) -> Vec<String> {
        vec![
            self.name
                .as_ref()
                .map_or(String::new(), |name| name.to_string()),
            self.column_name
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
            self.ref_table
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
            self.ref_column
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
        ]
    }
}

pub struct Index {
    name: Option<String>,
    column_name: Option<String>,
    r#type: Option<String>,
}

impl TableRow for Index {
    fn fields(&self) -> Vec<String> {
        vec![
            "name".to_string(),
            "column_name".to_string(),
            "type".to_string(),
        ]
    }

    fn columns(&self) -> Vec<String> {
        vec![
            self.name
                .as_ref()
                .map_or(String::new(), |name| name.to_string()),
            self.column_name
                .as_ref()
                .map_or(String::new(), |column_name| column_name.to_string()),
            self.r#type
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
        ]
    }
}

#[async_trait]
impl Pool for MySqlPool {
    async fn execute(&self, query: &str) -> anyhow::Result<ExecuteResult> {
        let query = query.trim();

        if query.to_uppercase().starts_with("SELECT") || query.to_uppercase().starts_with("SHOW") {
            let result = self.query(query).await?;
            return Ok(ExecuteResult::Read {
                headers: result.headers,
                rows: result.rows,
                database: Database {
                    name: "-".to_string(),
                    children: Vec::new(),
                },
                table: Table {
                    name: "-".to_string(),
                    create_time: None,
                    update_time: None,
                    engine: None,
                    schema: None,
                },
            });
        }

        let result = sqlx::query(query).execute(&self.pool).await?;
        Ok(ExecuteResult::Write {
            updated_rows: result.rows_affected(),
        })
    }

    async fn query(&self, query: &str) -> anyhow::Result<QueryResult> {
        let query = query.trim();

        if query.to_uppercase().starts_with("SELECT") || query.to_uppercase().starts_with("SHOW") {
            let mut rows = sqlx::query(query).fetch(&self.pool);
            let mut headers = vec![];
            let mut records = vec![];
            while let Some(row) = rows.try_next().await? {
              
                let mut new_row = vec![];
                for column in row.columns() {
                    let row = convert_column_value_to_string(&row, column)?;
                    new_row.push(row.0);
                    if records.len() == 0 { headers.push(row.1); };
                }
                records.push(new_row)
            }

            return Ok(QueryResult {
                headers,
                rows: records,
                updated_rows: 0,
            });
        }

        let result = sqlx::query(query).execute(&self.pool).await?;
        Ok(QueryResult {
            headers: vec![],
            rows: vec![],
            updated_rows: result.rows_affected(),
        })
    }

    async fn get_databases(&self) -> anyhow::Result<Vec<Database>> {
        let databases = sqlx::query("SHOW DATABASES")
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|table| table.get(0))
            .collect::<Vec<String>>();
        let mut list = vec![];
        for db in databases {
            list.push(Database::new(
                db.clone(),
                self.get_tables(db.clone()).await?,
            ))
        }
        Ok(list)
    }

    async fn get_tables(&self, database: String) -> anyhow::Result<Vec<Child>> {
        let query = format!("SHOW TABLE STATUS FROM `{}`", database);
        let mut rows = sqlx::query(query.as_str()).fetch(&self.pool);
        let mut tables = vec![];
        while let Some(row) = rows.try_next().await? {
            tables.push(Table {
                name: row.try_get("Name")?,
                create_time: row.try_get("Create_time")?,
                update_time: row.try_get("Update_time")?,
                engine: row.try_get("Engine")?,
                schema: None,
            })
        }
        Ok(tables.into_iter().map(|table| table.into()).collect())
    }

    async fn get_records(
        &self,
        database: &Database,
        table: &Table,
        page: u16,
        filter: Option<String>,
    ) -> anyhow::Result<(Vec<Header>, Vec<Vec<Value>>)> {
        let query = if let Some(filter) = filter {
            format!(
                "SELECT * FROM `{database}`.`{table}` WHERE {filter} LIMIT {page}, {limit}",
                database = database.name,
                table = table.name,
                filter = filter,
                page = page,
                limit = RECORDS_LIMIT_PER_PAGE
            )
        } else {
            format!(
                "SELECT * FROM `{}`.`{}` LIMIT {page}, {limit}",
                database.name,
                table.name,
                page = page,
                limit = RECORDS_LIMIT_PER_PAGE
            )
        };
        let mut rows = sqlx::query(query.as_str()).fetch(&self.pool);
        let mut headers = vec![];
        let mut records = vec![];
        while let Some(row) = rows.try_next().await? {
            // headers = row
            //     .columns()
            //     .iter()
            //     .map(|column| column.name().to_string())
            //     .collect();
            let mut new_row = vec![];
            for column in row.columns() {
                let row = convert_column_value_to_string(&row, column)?;
                new_row.push(row.0);
                if records.len() == 0 { headers.push(row.1); };
            }
            records.push(new_row)
        }
        Ok((headers, records))
    }

    async fn get_columns(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let query = format!(
            "SHOW FULL COLUMNS FROM `{}`.`{}`",
            database.name, table.name
        );
        let mut rows = sqlx::query(query.as_str()).fetch(&self.pool);
        let mut columns: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            columns.push(Box::new(Column {
                name: row.try_get("Field")?,
                r#type: row.try_get("Type")?,
                null: row.try_get("Null")?,
                default: row.try_get("Default")?,
                comment: row.try_get("Comment")?,
            }))
        }
        Ok(columns)
    }

    async fn get_constraints(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let mut rows = sqlx::query(
            "
        SELECT
            COLUMN_NAME,
            CONSTRAINT_NAME
        FROM
            information_schema.KEY_COLUMN_USAGE
        WHERE
            REFERENCED_TABLE_SCHEMA IS NULL
            AND REFERENCED_TABLE_NAME IS NULL
            AND TABLE_SCHEMA = ?
            AND TABLE_NAME = ?
        ",
        )
        .bind(&database.name)
        .bind(&table.name)
        .fetch(&self.pool);
        let mut constraints: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            constraints.push(Box::new(Constraint {
                name: row.try_get("CONSTRAINT_NAME")?,
                column_name: row.try_get("COLUMN_NAME")?,
            }))
        }
        Ok(constraints)
    }

    async fn get_foreign_keys(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let mut rows = sqlx::query(
            "
        SELECT
            TABLE_NAME,
            COLUMN_NAME,
            CONSTRAINT_NAME,
            REFERENCED_TABLE_SCHEMA,
            REFERENCED_TABLE_NAME,
            REFERENCED_COLUMN_NAME
        FROM
            INFORMATION_SCHEMA.KEY_COLUMN_USAGE
        WHERE
            REFERENCED_TABLE_SCHEMA IS NOT NULL
            AND REFERENCED_TABLE_NAME IS NOT NULL
            AND TABLE_SCHEMA = ?
            AND TABLE_NAME = ?
        ",
        )
        .bind(&database.name)
        .bind(&table.name)
        .fetch(&self.pool);
        let mut foreign_keys: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            foreign_keys.push(Box::new(ForeignKey {
                name: row.try_get("CONSTRAINT_NAME")?,
                column_name: row.try_get("COLUMN_NAME")?,
                ref_table: row.try_get("REFERENCED_TABLE_NAME")?,
                ref_column: row.try_get("REFERENCED_COLUMN_NAME")?,
            }))
        }
        Ok(foreign_keys)
    }

    async fn get_indexes(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let mut rows = sqlx::query(
            "
        SELECT
            DISTINCT TABLE_NAME,
            INDEX_NAME,
            INDEX_TYPE,
            COLUMN_NAME
        FROM
            INFORMATION_SCHEMA.STATISTICS
        WHERE
            TABLE_SCHEMA = ?
            AND TABLE_NAME = ?
        ",
        )
        .bind(&database.name)
        .bind(&table.name)
        .fetch(&self.pool);
        let mut foreign_keys: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            foreign_keys.push(Box::new(Index {
                name: row.try_get("INDEX_NAME")?,
                column_name: row.try_get("COLUMN_NAME")?,
                r#type: row.try_get("INDEX_TYPE")?,
            }))
        }
        Ok(foreign_keys)
    }

    async fn close(&self) {
        self.pool.close().await;
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::MySql
    }
}

fn convert_column_value_to_string(row: &MySqlRow, column: &MySqlColumn) -> anyhow::Result<(Value, Header)> {
    let column_name = column.name();

    if let Ok(value) = row.try_get(column_name) {
        let value: Option<String> = value;
        let header = Header::new(column_name.to_string(), ColType::VarChar);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<&str> = value;
        let header = Header::new(column_name.to_string(), ColType::VarChar);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i8> = value;
        let header = Header::new(column_name.to_string(), ColType::Int);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i16> = value;
        let header = Header::new(column_name.to_string(), ColType::Int);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i32> = value;
        let header = Header::new(column_name.to_string(), ColType::Int);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i64> = value;
        let header = Header::new(column_name.to_string(), ColType::Int);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<f32> = value;
        let header = Header::new(column_name.to_string(), ColType::Float);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<f64> = value;
        let header = Header::new(column_name.to_string(), ColType::Float);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<u8> = value;
        let header = Header::new(column_name.to_string(), ColType::Int);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<u16> = value;
        let header = Header::new(column_name.to_string(), ColType::Int);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<u32> = value;
        let header = Header::new(column_name.to_string(), ColType::Int);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<u64> = value;
        let header = Header::new(column_name.to_string(), ColType::Int);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<rust_decimal::Decimal> = value;
        let header = Header::new(column_name.to_string(), ColType::Int);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<NaiveDate> = value;
        let header = Header::new(column_name.to_string(), ColType::Date);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<NaiveTime> = value;
        let header = Header::new(column_name.to_string(), ColType::Date);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<NaiveDateTime> = value;
        let header = Header::new(column_name.to_string(), ColType::Date);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::DateTime<chrono::Utc>> = value;
        let header = Header::new(column_name.to_string(), ColType::Date);
        let t = value.map(|t| t.to_string().strip_suffix(" UTC").unwrap().to_string());
        Ok((get_or_null!(t), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<serde_json::Value> = value;
        let header = Header::new(column_name.to_string(), ColType::Date);
        Ok((get_or_null!(value), header))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<bool> = value;
        let header = Header::new(column_name.to_string(), ColType::Boolean);
        Ok((get_or_null!(value), header))
    } else {
        let index = column.ordinal();
        if let Ok(val) = row.try_get_raw(index) {
            // https://docs.rs/sqlx-core/0.5.11/src/sqlx_core/mysql/types/str.rs.html
            // match val.format() {
            //     MySqlValueFormat::Binary => {},
            //     MySqlValueFormat::Text => {},
            // }
            if let Ok(value) = Decode::<MySql>::decode(val) {
                let value: &str = value;
                let header = Header::new(column_name.to_string(), ColType::VarChar);
                return Ok((Value::new(value.to_string()), header))
            }
        }
        anyhow::bail!(
            "column type not implemented: `{}` {}",
            column_name,
            column.type_info().clone().name()
        )
    }
}
