use std::collections::HashMap;
use regex::{escape, RegexBuilder};
use crate::sql::Completion;
use crate::config::{DatabaseType};
use super::{suggest_type, SuggestType, last_word};

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

fn find_matches(text: &str, collection: &Vec<&str>, start_only: bool, fuzzy: bool) -> Vec<String> {
    let last = last_word(text, "most_punctuations");
    let mut completions = vec![];
    let s = if fuzzy { last.chars().map(|c| format!(".*?{}", escape(&c.to_string()))).collect()} 
    else if start_only { format!(r"^{}", escape(last)) }
    else { format!(r".*{}", escape(last)) };
    let reg = RegexBuilder::new(&s).case_insensitive(true).build().unwrap();
    for word in collection {
        if reg.is_match(word) {
            completions.push(word);
        }
    }
    let is_upper = last.chars().last().map(|c| c.is_uppercase()).unwrap_or(false);
    let mut completions = completions.iter().map(|w| if is_upper { w.to_uppercase() } else { w.to_lowercase() }).collect::<Vec<_>>();
    completions.sort();
    completions
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
                _ => {}
           }
        }
        completions
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