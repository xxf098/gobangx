use std::time::Duration;
use sqlx::mssql::{Mssql, MssqlColumn, MssqlRow};
use sqlx::{Column as _, Row as _, TypeInfo as _};
use async_trait::async_trait;
use database_tree::{Child, Database, Table, Schema};
use futures::TryStreamExt;
use itertools::Itertools;
use super::{ExecuteResult, QueryResult, Pool, TableRow, RECORDS_LIMIT_PER_PAGE, ColType, Header, Value};
use crate::get_or_null;
use crate::config::DatabaseType;


pub type MssqlPoolOptions = sqlx::pool::PoolOptions<Mssql>;


pub struct MssqlPool {
    pool: sqlx::mssql::MssqlPool,
}

impl MssqlPool {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool: MssqlPoolOptions::new()
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
impl Pool for MssqlPool {

    async fn execute(&self, query: &str) -> anyhow::Result<ExecuteResult> {
        let query = query.trim();
        if query.to_uppercase().starts_with("SELECT") {
            let mut rows = sqlx::query(query).fetch(&self.pool);
            let mut headers = vec![];
            let mut records = vec![];
            while let Some(row) = rows.try_next().await? {
                // headers = row
                //     .columns()
                //     .iter()
                //     .map(|column| column.name().to_string())
                //     .collect();
                let mut new_row = vec![];
                for column in row.columns().iter() {
                    let row = convert_column_value_to_string(&row, column)?;
                    new_row.push(row.0);
                    if records.len() == 0 { headers.push(row.1); }
                }
                records.push(new_row)
            }
            return Ok(ExecuteResult::Read {
                headers,
                rows: records,
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

    async fn query(&self, _query: &str) -> anyhow::Result<QueryResult> {
        unimplemented!()
    }

    async fn get_databases(&self) -> anyhow::Result<Vec<Database>> {
        let databases = sqlx::query("select name from sys.databases")
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
        let query = format!("SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_CATALOG = '{}'", database.to_uppercase());
        let mut rows = sqlx::query(query.as_str()).fetch(&self.pool);
        let mut tables = Vec::new();
        while let Some(row) = rows.try_next().await? {
            tables.push(Table {
                name: row.try_get("TABLE_NAME")?,
                create_time: None,
                update_time: None,
                engine: None,
                schema: row.try_get("TABLE_SCHEMA")?,
            })
        }
        let mut schemas = vec![];
        for (key, group) in &tables
            .iter()
            .sorted_by(|a, b| Ord::cmp(&b.schema, &a.schema))
            .group_by(|t| t.schema.as_ref())
        {
            if let Some(key) = key {
                schemas.push(
                    Schema {
                        name: key.to_string(),
                        tables: group.cloned().collect(),
                    }
                    .into(),
                )
            }
        }
        Ok(schemas)
    }

    async fn get_records(
        &self,
        database: &Database,
        table: &Table,
        _page: u16,
        filter: Option<String>,
        _orderby: Option<String>,
    ) -> anyhow::Result<(Vec<Header>, Vec<Vec<Value>>)> {
        // FIXME
        let query = if let Some(filter) = filter.as_ref() {
            format!(
                r#"SELECT TOP {limit} * FROM "{database}"."{table_schema}"."{table}" WHERE {filter}"#,
                database = database.name,
                table = table.name,
                filter = filter,
                table_schema = table.schema.clone().unwrap_or_else(|| "public".to_string()),
                limit = RECORDS_LIMIT_PER_PAGE
            )
        } else {
            format!(
                r#"SELECT TOP {limit} * FROM "{database}"."{table_schema}"."{table}""#,
                database = database.name,
                table = table.name,
                table_schema = table.schema.clone().unwrap_or_else(|| "public".to_string()),
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
            //     .map(|column| Header::new(column.name().to_string()))
            //     .collect();
            let mut new_row = vec![];
            for column in row.columns().iter() {
                let row = convert_column_value_to_string(&row, column)?;
                new_row.push(row.0);
                headers.push(row.1);
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
        let table_schema = table
            .schema
            .as_ref()
            .map_or("public", |schema| schema.as_str());
        let query = format!("SELECT * FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_CATALOG = '{}' AND table_schema = '{}' AND table_name = '{}'", &database.name, table_schema, &table.name);
        let mut rows = sqlx::query(query.as_str())
            .fetch(&self.pool);
        let mut columns: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            columns.push(Box::new(Column {
                name: row.try_get("COLUMN_NAME")?,
                r#type: row.try_get("DATA_TYPE")?,
                // null: row.try_get("IS_NULLABLE")?, // FIXME
                null: None,
                default: row.try_get("COLUMN_DEFAULT")?,
                comment: None,
            }))
        }
        Ok(columns)
    }

    async fn get_constraints(
        &self,
        _database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let mut rows = sqlx::query(
            "
        SELECT
            tc.table_schema,
            tc.constraint_name,
            tc.table_name,
            kcu.column_name,
            ccu.table_schema AS foreign_table_schema,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name
        FROM
            information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu ON ccu.constraint_name = tc.constraint_name
            AND ccu.table_schema = tc.table_schema
        WHERE
            NOT tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_name = '$1'
        ",
        )
        .bind(&table.name)
        .fetch(&self.pool);
        let mut constraints: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            constraints.push(Box::new(Constraint {
                name: row.try_get("constraint_name")?,
                column_name: row.try_get("column_name")?,
            }))
        }
        Ok(constraints)        

    }

    async fn get_foreign_keys(
        &self,
        _database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let mut rows = sqlx::query(
            "
        SELECT
            tc.table_schema,
            tc.constraint_name,
            tc.table_name,
            kcu.column_name,
            ccu.table_schema AS foreign_table_schema,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name
        FROM
            information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu ON ccu.constraint_name = tc.constraint_name
            AND ccu.table_schema = tc.table_schema
        WHERE
            tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_name = '$1'
        ",
        )
        .bind(&table.name)
        .fetch(&self.pool);
        let mut constraints: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            constraints.push(Box::new(ForeignKey {
                name: row.try_get("constraint_name")?,
                column_name: row.try_get("column_name")?,
                ref_table: row.try_get("foreign_table_name")?,
                ref_column: row.try_get("foreign_column_name")?,
            }))
        }
        Ok(constraints)        
    }

    async fn get_indexes(
        &self,
        _database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let mut rows = sqlx::query(
            "
        SELECT
            TABLENAME = t.name,
            INDEXNAME = ind.name,
            INDEXID = ind.index_id,
            TYPENAME = ind.type_desc,
            COLUMNID = ic.index_column_id,
            COLUMNNAME = col.name
       FROM
            sys.indexes ind
       INNER JOIN
            sys.index_columns ic ON  ind.object_id = ic.object_id and ind.index_id = ic.index_id
       INNER JOIN
            sys.columns col ON ic.object_id = col.object_id and ic.column_id = col.column_id
       INNER JOIN
            sys.tables t ON ind.object_id = t.object_id
       WHERE t.name = '$1';
        ",
        )
        .bind(&table.name)
        .fetch(&self.pool);
        let mut foreign_keys: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            foreign_keys.push(Box::new(Index {
                name: row.try_get("INDEXNAME")?,
                column_name: row.try_get("COLUMNNAME")?,
                r#type: row.try_get("TYPENAME")?,
            }))
        }
        Ok(foreign_keys)
    }

    async fn close(&self) {
        self.pool.close().await;
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::Mssql
    }
}


fn convert_column_value_to_string(row: &MssqlRow, column: &MssqlColumn) -> anyhow::Result<(Value, Header)> {
    let column_name = column.name();

    if let Ok(value) = row.try_get(column_name) {
        let value: Option<String> = value;
        let header = Header::new(column_name.to_string(), ColType::VarChar);
        Ok((get_or_null!(value), header))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<&str> = value;
    //     Ok(get_or_null!(value))
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
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<u8> = value;
    //     Ok(get_or_null!(value))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<u16> = value;
    //     Ok(get_or_null!(value))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<u32> = value;
    //     Ok(get_or_null!(value))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<u64> = value;
    //     Ok(get_or_null!(value))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<rust_decimal::Decimal> = value;
    //     Ok(get_or_null!(value))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<NaiveDate> = value;
    //     Ok(get_or_null!(value))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<NaiveTime> = value;
    //     Ok(get_or_null!(value))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<NaiveDateTime> = value;
    //     Ok(get_or_null!(value))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<chrono::DateTime<chrono::Utc>> = value;
    //     Ok(get_or_null!(value))
    // } else if let Ok(value) = row.try_get(column_name) {
    //     let value: Option<serde_json::Value> = value;
    //     Ok(get_or_null!(value))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<bool> = value;
        let header = Header::new(column_name.to_string(), ColType::Boolean);
        Ok((get_or_null!(value), header))
    } else {
        anyhow::bail!(
            "column type not implemented: `{}` {}",
            column_name,
            column.type_info().clone().name()
        )
    }
}
