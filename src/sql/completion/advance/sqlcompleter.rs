use std::rc::Rc;
use std::cell::RefCell;
use regex::{escape, RegexBuilder};
use crate::sql::{Completion, DbMetadata};
use crate::config::{DatabaseType};
use super::{suggest_type, last_word, SuggestType, SuggestTable};

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
    users: Vec<String>,
    show_items: Vec<String>,
    dbname: String,
    dbmetadata: Rc<RefCell<DbMetadata>>,
    all_completions: Vec<String>,
    keywords: Vec<&'static str>,
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

    pub fn reset_completions(&mut self) {
        // self.databases = vec![];
        self.users = vec![];
        self.show_items = vec![];
        self.dbname = "".to_string();
        self.dbmetadata = Rc::new(RefCell::new(DbMetadata::default()));
        self.all_completions = self.keywords.iter().map(|k| k.to_string()).collect::<Vec<_>>();
    }

    fn populate_scoped_cols(&self, scoped_tbls: &Vec<SuggestTable>) -> Vec<String> {
        let meta = self.dbmetadata.borrow();
        let mut columns = vec![];
        for tbl in scoped_tbls {
            let schema = tbl.schema.clone().unwrap_or(self.dbname.clone());
            let relname = tbl.table.clone();
            // let escaped_relname = "";
            let key = (schema, relname);
            if let Some(cols) = meta.tables.get(&key) {
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
    fn populate_schema_objects(&self, schema: &str, obj_type: &str) -> Vec<String> {
        match obj_type {
            "tables" => {
                let tables = &self.dbmetadata.borrow().tables;
                tables.iter().filter_map(|(k, _)| if k.0 == schema { Some(k.1.clone()) } else { None } ).collect()
            },
            _ => vec![],
        }
    }

    fn get_completions(&self, full_text: &str) -> Vec<String>{
        let word_before_cursor = full_text;
        let suggestions = suggest_type(full_text, full_text);
        let mut completions= vec![];
        for suggestion in suggestions {
           match suggestion {
                SuggestType::Keyword => {
                    let keywords = find_matches(word_before_cursor, &self.keywords, true, false);
                    completions.extend(keywords);
                },
                SuggestType::Column(tables) => {
                    let scoped_cols = self.populate_scoped_cols(&tables);
                    let cols = find_matches(word_before_cursor, &scoped_cols, false, true);
                    completions.extend(cols)
                },
                SuggestType::Table(schema) => {
                    let tables = self.populate_schema_objects(&schema, "tables");
                    let tables = find_matches(word_before_cursor, &tables, false, true);
                    completions.extend(tables)
                },
                _ => {}
           }
        }
        completions
    }
}

impl Completion for AdvanceSQLCompleter {
    fn new(db_type: DatabaseType, _candidates: Vec<String>) -> Self {
        let dbmetadata = Rc::new(RefCell::new(DbMetadata::default()));
        let keywords: Vec<_> = db_type.clone().into();
        let all_completions = keywords.iter().map(|k| k.to_string()).collect::<Vec<_>>();
        AdvanceSQLCompleter{
            // databases: vec![],
            users: vec![],
            show_items: vec![],
            dbname: "".to_string(),
            dbmetadata: dbmetadata,
            all_completions: all_completions,
            keywords: keywords,
            // functions: FUNCTIONS.to_vec(),
        }
    }

    fn complete(&self, full_text: &str) -> Vec<String> {
        self.get_completions(full_text)
    }

    fn update(&mut self, _candidates: &[String], db_metadata: Option<Rc<RefCell<DbMetadata>>>) {
        if let Some(db_metadata) = db_metadata {
            self.dbname = db_metadata.borrow().dbname.clone();
            self.dbmetadata = db_metadata;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_keyword() {
        let completer = AdvanceSQLCompleter::new(DatabaseType::MySql, vec![]);
        let completions = completer.get_completions("s");
        println!("{:?}", completions);
    }
}