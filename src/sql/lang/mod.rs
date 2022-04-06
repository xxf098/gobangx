mod standard;
mod mysql;
mod postgresql;

pub use standard::Standard;
pub use mysql::MySQL;
pub use postgresql::PostgreSQL;

use std::convert::TryFrom;
use crate::config::DatabaseType;
use crate::sql::token::tokenizer::{Tokenizer, Tokenize};



impl TryFrom<DatabaseType> for Tokenizer {

    type Error = anyhow::Error;

    fn try_from(value: DatabaseType) -> Result<Self, Self::Error> {
        match value {
            DatabaseType::Sqlite => Standard{}.tokenizer(),
            DatabaseType::Postgres => PostgreSQL{}.tokenizer(),
            DatabaseType::MySql => MySQL{}.tokenizer(),
            DatabaseType::Mssql => Standard{}.tokenizer()
        }
    }
}



