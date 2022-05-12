use std::sync::{Arc, RwLock};
use itertools::Itertools;
use regex::{escape, RegexBuilder};
use crate::sql::{Completion, DbMetadata};
use crate::config::{DatabaseType};
use super::{last_word, SuggestType, SuggestTable, Suggest};

/*
const KEYWORDS: [&str; 134] = ["ACCESS","ADD","ALL","ALTER TABLE","AND","ANY","AS",
        "ASC","AUTO_INCREMENT","BEFORE","BEGIN","BETWEEN",
        "BIGINT","BINARY","BY","CASE","CHANGE MASTER TO","CHAR",
        "CHARACTER SET","CHECK","COLLATE","COLUMN","COMMENT",
        "COMMIT","CONSTRAINT","CREATE","CURRENT",
        "CURRENT_TIMESTAMP","DATABASE","DATE","DECIMAL","DEFAULT",
        "DELETE FROM","DESC","DESCRIBE","DROP",
        "ELSE","END","ENGINE","ESCAPE","EXISTS","FILE","FLOAT",
        "FOR","FOREIGN KEY","FORMAT","FROM","FULL","FUNCTION",
        "GRANT","GROUP BY","HAVING","HOST","IDENTIFIED","IN",
        "INCREMENT","INDEX","INSERT INTO","INT","INTEGER",
        "INTERVAL","INTO","IS","JOIN","KEY","LEFT","LEVEL",
        "LIKE","LIMIT","LOCK","LOGS","LONG","MASTER",
        "MEDIUMINT","MODE","MODIFY","NOT","NULL","NUMBER",
        "OFFSET","ON","OPTION","OR","ORDER BY","OUTER","OWNER",
        "PASSWORD","PORT","PRIMARY","PRIVILEGES","PROCESSLIST",
        "PURGE","REFERENCES","REGEXP","RENAME","REPAIR","RESET",
        "REVOKE","RIGHT","ROLLBACK","ROW","ROWS","ROW_FORMAT",
        "SAVEPOINT","SELECT","SESSION","SET","SHARE","SHOW",
        "SLAVE","SMALLINT","SMALLINT","START","STOP","TABLE",
        "THEN","TINYINT","TO","TRANSACTION","TRIGGER","TRUNCATE",
        "UNION","UNIQUE","UNSIGNED","UPDATE","USE","USER",
        "USING","VALUES","VARCHAR","VIEW","WHEN","WHERE","WITH"];

const FUNCTIONS: [&str; 19] = ["AVG","CONCAT","COUNT","DISTINCT","FIRST","FORMAT",
        "FROM_UNIXTIME","LAST","LCASE","LEN","MAX","MID",
        "MIN","NOW","ROUND","SUM","TOP","UCASE","UNIX_TIMESTAMP"];
*/

// TODO: &str
pub struct AdvanceSQLCompleter {
    // databases: Vec<String>,
    // db_type: DatabaseType,
    // users: Vec<String>,
    show_items: Vec<String>,
    dbname: String,
    dbmetadata: Arc<RwLock<DbMetadata>>,
    // all_completions: Vec<String>,
    keywords: Vec<&'static str>,
    suggest: Suggest,
    // functions: Vec<&'static str>,
}

fn find_matches<T: AsRef<str>>(text: &str, collection: &[T], start_only: bool, fuzzy: bool) -> Vec<String> {
    let last = last_word(text, "most_punctuations");
    let mut completions = vec![];
    let s = if fuzzy { last.chars().map(|c| format!(".*?{}", escape(&c.to_string()))).collect()} 
    else if start_only { format!(r"^{}", escape(last)) }
    else { format!(r".*{}", escape(last)) };
    let reg = RegexBuilder::new(&s).case_insensitive(true).build().unwrap();
    for word in collection {
        if reg.is_match(word.as_ref()) {
            completions.push(word);
        }
    }
    let is_upper = last.chars().last().map(|c| c.is_uppercase()).unwrap_or(false);
    let mut completions = completions.iter().map(|w| if is_upper { w.as_ref().to_uppercase() } else { w.as_ref().to_lowercase() }).collect::<Vec<_>>();
    completions.sort();
    completions
}

impl AdvanceSQLCompleter {

    pub fn _reset_completions(&mut self) {
        // self.databases = vec![];
        // self.users = vec![];
        self.show_items = vec![];
        self.dbname = "".to_string();
        self.dbmetadata = Arc::new(RwLock::new(DbMetadata::default()));
        // self.all_completions = self.keywords.iter().map(|k| k.to_string()).collect::<Vec<_>>();
    }

    fn populate_scoped_cols(&self, scoped_tbls: &Vec<SuggestTable>) -> Vec<String> {
        let meta = self.dbmetadata.read().unwrap();
        let mut columns = vec![];
        for tbl in scoped_tbls {
            let schema = tbl.schema.clone().unwrap_or(self.dbname.clone());
            let relname = tbl.table.clone();
            // let escaped_relname = "";
            let key = (schema, relname);
            if let Some(cols) = meta.columns.get(&key) {
                // FIXME: use ref
                columns.extend(cols.clone());
                continue;
            }
            if let Some(cols) = meta.views.get(&key) {
                // FIXME: use ref
                columns.extend(cols.clone());
                continue;
            }
        }
        return columns
    }

    // get table names
    // schema: schema name
    // obj_type: tables, views
    fn populate_schema_objects(&self, _schema: &str, obj_type: &str) -> Vec<String> {
        match obj_type {
            "tables" => {
                let tables = &self.dbmetadata.read().unwrap().tables;
                tables.clone()
            },
            "schemas" => {
                let schemas = &self.dbmetadata.read().unwrap().schemas;
                schemas.clone()
            },
            "databases" => {
                let databases = &self.dbmetadata.read().unwrap().databases;
                databases.clone()
            },
            _ => vec![],
        }
    }

    fn get_completions(&self, full_text: &str) -> Vec<String>{
        let word_before_cursor = full_text;
        let suggestions = self.suggest.suggest_type(full_text, full_text);
        // let suggestions = vec![SuggestType::Keyword, SuggestType::Column(vec![SuggestTable::new(Some("main"), "logs", None)])];
        let mut completions= vec![];
        for suggestion in suggestions {
           match suggestion {
                SuggestType::Column(tables) => {
                    let cols = if tables.is_empty() {
                        let meta = self.dbmetadata.read().unwrap();
                        let mut scoped_cols = vec![];
                        meta.columns.iter().for_each(|(_, v)| scoped_cols.extend(v));
                        find_matches(word_before_cursor, &scoped_cols, false, true)
                    } else {
                        let scoped_cols = self.populate_scoped_cols(&tables);
                        find_matches(word_before_cursor, &scoped_cols, false, true)
                    };
                    completions.extend(cols)
                },
                SuggestType::Table(schema) => {
                    let tables = self.populate_schema_objects(&schema, "tables");
                    let tables = find_matches(word_before_cursor, &tables, false, true);
                    completions.extend(tables)
                },
                SuggestType::Database => {
                    let databases = self.populate_schema_objects("", "databases");
                    let databases = find_matches(word_before_cursor, &databases, false, true);
                    completions.extend(databases)
                },
                SuggestType::Keyword => {
                    let keywords = find_matches(word_before_cursor, &self.keywords, true, false);
                    completions.extend(keywords);
                },
                SuggestType::Show => {
                    let show_items = find_matches(word_before_cursor, &self.show_items, false, true);
                    completions.extend(show_items);
                }
                _ => {}
           }
        }
        completions.into_iter().unique().collect()
    }
}

impl Completion for AdvanceSQLCompleter {
    fn new(db_type: DatabaseType, _candidates: Vec<String>) -> Self {
        let dbmetadata = Arc::new(RwLock::new(DbMetadata::default()));
        let keywords: Vec<_> = db_type.clone().into();
        // let all_completions = keywords.iter().map(|k| k.to_string()).collect::<Vec<_>>();
        AdvanceSQLCompleter{
            // databases: vec![],
            // users: vec![],
            show_items: vec![],
            dbname: "".to_string(),
            dbmetadata: dbmetadata,
            // all_completions: all_completions,
            keywords: keywords,
            suggest: Suggest::default(), // TODO: init background
            // functions: FUNCTIONS.to_vec(),
        }
    }

    fn complete(&self, full_text: &str) -> Vec<String> {
        self.get_completions(full_text)
    }

    fn update(&mut self, _candidates: &[String], db_metadata: Option<Arc<RwLock<DbMetadata>>>) {
        if let Some(db_metadata) = db_metadata {
            self.dbname = db_metadata.read().unwrap().current_db.clone();
            self.dbmetadata = db_metadata;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;


    #[test]
    fn test_get_keyword() {
        let completer = AdvanceSQLCompleter::new(DatabaseType::MySql, vec![]);
        let completions = completer.get_completions("sel");
        println!("{:?}", completions);
    }

    #[test]
    fn test_suggest_type1() {
        let suggest = Suggest::default();
        let now = Instant::now();
        let full_text = "select l";
        let suggestions = suggest.suggest_type(full_text, full_text);
        let elapsed = now.elapsed();
        println!("elapsed: {:?}ms", elapsed.as_millis());
        println!("{:?}", suggestions)
    }

    #[test]
    fn test_suggest_type2() {
        let suggest = Suggest::default();
        let now = Instant::now();
        let full_text = "select id,";
        let suggestions = suggest.suggest_type(full_text, full_text);
        let elapsed = now.elapsed();
        println!("elapsed: {:?}ms", elapsed.as_millis());
        println!("{:?}", suggestions)
    }
}