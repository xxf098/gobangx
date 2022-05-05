use std::collections::HashMap;
use crate::sql::Completion;
use crate::config::{DatabaseType};
use super::completion_engine::Engine;

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

pub fn get_completions() {
}

struct DbMetadata {
    tables: HashMap<String, Vec<String>>,
}

// TODO: &str
pub struct AdvanceSQLCompleter {
    databases: Vec<String>,
    users: Vec<String>,
    show_items: Vec<String>,
    dbname: String,
    dbmetadata: DbMetadata,
    all_completions: Vec<String>,
    keywords: Vec<&'static str>,
    functions: Vec<&'static str>,
}

impl AdvanceSQLCompleter {

    fn reset_completions(&mut self) {
        self.databases = vec![];
        self.users = vec![];
        self.show_items = vec![];
        self.dbname = "".to_string();
        self.dbmetadata = DbMetadata{ tables: HashMap::new() };
        let mut all_completions = KEYWORDS.to_vec();
        all_completions.extend(FUNCTIONS.to_vec());
        self.all_completions = all_completions.into_iter().map(|k| k.to_string()).collect();
    }
}

impl Completion for AdvanceSQLCompleter {
    fn new(db_type: DatabaseType, candidates: Vec<String>) -> Self {
        let dbmetadata = DbMetadata{
            tables: HashMap::new(),
        };
        let mut all_completions = KEYWORDS.to_vec();
        all_completions.extend(FUNCTIONS.to_vec());
        AdvanceSQLCompleter{
            databases: vec![],
            users: vec![],
            show_items: vec![],
            dbname: "".to_string(),
            dbmetadata: dbmetadata,
            all_completions: all_completions.into_iter().map(|k| k.to_string()).collect(),
            keywords: KEYWORDS.to_vec(),
            functions: FUNCTIONS.to_vec(),
        }
    }

    fn complete(&self, full_text: String, word: &String) -> Vec<&String> {
        vec![]
    }

    fn update_candidates(&mut self, candidates: &[String]) {

    }
}