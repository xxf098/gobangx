use std::collections::HashMap;
use std::cell::RefCell;
use super::{last_word, find_prev_keyword, extract_tables};
use sqlparse::{ Token, TokenType, TokenList, Parser };

#[derive(Debug, Clone, PartialEq)]
pub enum SuggestType {
    Keyword,
    Special,
    Database,
    Schema(Option<String>), // database name
    Table(String), // schema name
    View(String), // schema name
    Column(Vec<SuggestTable>),
    Function(String),
    Alias(Vec<String>),
    Show,
    Change,
    User,
    TableFormat,
}

impl SuggestType {

    pub fn column(schema: Option<&str>, table: &str, alias: Option<&str>) -> SuggestType {
        SuggestType::Column(vec![SuggestTable::new(schema, table, alias)])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuggestTable {
    pub schema: Option<String>,
    pub table: String,
    pub alias: Option<String>
}

impl SuggestTable {

    pub fn new(schema: Option<&str>, table: &str, alias: Option<&str>) -> Self {
        Self { 
            schema: schema.map(|s| s.to_string()), 
            table: table.to_string(), 
            alias: alias.map(|s| s.to_string()),
        }
    }

    pub fn identifies(&self, id: &str) -> bool {
        self.alias.as_deref() == Some(id) || 
        self.table == id ||
        self.schema.as_ref().map(|s| format!("{}.{}", s, self.table)).as_deref() == Some(id)
    }
}

#[derive(Default)]
pub struct Suggest {
    parser: Parser,
    cache: RefCell<HashMap<String, Vec<Token>>>,
}

impl Suggest {

    fn parse(&self, sql: &str) -> Vec<Token> {
        let mut cache = self.cache.borrow_mut();
        if let Some(tokens) = cache.get(sql) {
            tokens.clone()
        } else {
            let tokens = self.parser.parse(sql);
            cache.insert(sql.to_string(), tokens.clone());
            tokens
        }
    }
    
    // TODO: support multiple statement
    pub fn suggest_type(&self, full_text: &str, text_before_cursor: &str) -> Vec<SuggestType> {
        let word_before_cursor = last_word(text_before_cursor, "many_punctuations");
        let mut identifier: Option<&Token> = None;
        // FIXME: clone
        let mut tokens = vec![];
        
        let parsed: Vec<Token> =  if !word_before_cursor.is_empty() {
            if word_before_cursor.ends_with("(") {
                self.parse(text_before_cursor)
            } else {
                tokens = self.parse(word_before_cursor);
                let p = &tokens[0];
                let l = text_before_cursor.len() - word_before_cursor.len();
                if p.children.len() > 0 && p.children.token_idx(Some(0)).map(|t| t.typ == TokenType::Identifier).unwrap_or(false) {
                    identifier = Some(p);
                }
                self.parse(&text_before_cursor[..l])
            }
        } else {
            self.parse(text_before_cursor)
        };

    
        let statement = TokenList::new(parsed);
    
        let last_token_idx = statement.token_prev(statement.len(), true);
        let last_token = statement.token_idx(last_token_idx);
   
    
        let suggests = self.suggest_based_on_last_token(last_token, None, text_before_cursor, full_text, identifier);
        suggests
    }

    pub fn suggest_based_on_last_token(&self, token: Option<&Token>, token_str: Option<&str>, text_before_cursor: &str, full_text: &str, identifier: Option<&Token>) -> Vec<SuggestType> {
        if token.is_none() && token_str.is_none() {
            return vec![SuggestType::Keyword, SuggestType::Special]
        }
        let mut token_v = token_str.map(|s| s.to_string()).unwrap_or("".to_string());
        if let Some(token) = token {
            if token.typ == TokenType::Comparison {
                let t = token.children.token_idx(Some(token.children.len()-1));
                token_v = t.unwrap().value.to_lowercase();
            } else if token.typ == TokenType::Where {
                let (prev_keyword, text_before_cursor) = find_prev_keyword(text_before_cursor, &self.parser);
                if !prev_keyword.as_ref().map(|t| t.typ == TokenType::Where).unwrap_or(false) {
                    return self.suggest_based_on_last_token(prev_keyword.as_ref(), None, &text_before_cursor, full_text, identifier);
                }
            } else {
                token_v = token.value.to_lowercase();
            }
        }
        match token_v.as_ref() {
            v if v.ends_with("(") => {
                let p = self.parse(text_before_cursor);
                // Get the token before the parens
                let p = TokenList::new(p);
                // Four possibilities:
                if p.token_idx(Some(p.len()-1)).map(|t| t.typ == TokenType::Where).unwrap_or(false) {
                    let token = p.token_idx(Some(p.len()-1));
                    let column_suggestions = self.suggest_based_on_last_token(None, Some("where"), text_before_cursor, full_text, identifier);
                    // Check for a subquery expression 
                    let where_tokenlist = &token.unwrap().children;
                    let pidx = where_tokenlist.token_prev(where_tokenlist.len()-1, true);
                    let mut ptoken = where_tokenlist.token_idx(pidx);
                    if ptoken.map(|t| t.typ == TokenType::Comparison).unwrap_or(false) {
                        // e.g. "SELECT foo FROM bar WHERE foo = ANY("
                        let children = &ptoken.unwrap().children;
                        ptoken = children.token_idx(Some(children.len()-1));
                    }
                    if ptoken.map(|t| t.value.to_lowercase() == "exists").unwrap_or(false) {
                        return vec![SuggestType::Keyword]
                    } else {
                        return column_suggestions
                    }
                }
                let idx = p.token_prev(p.len()-1, true);
                let ptoken = p.token_idx(idx);
                if ptoken.map(|t| t.value.to_lowercase() == "using").unwrap_or(false) {
                    // tbl1 INNER JOIN tbl2 USING (col1, col2)
                    let tables = extract_tables(full_text, &self.parser);
                    // suggest columns that are present in more than one table
                    // FIXME: drop_unique
                    return vec![SuggestType::Column(tables)]
                } else if p.token_idx(Some(0)).map(|t| t.value.to_lowercase() == "select").unwrap_or(false) {
                    // If the lparen is preceeded by a space chances are we're about to do a sub-select.
                    if last_word(text_before_cursor, "all_punctuations").starts_with("(") {
                        return vec![SuggestType::Keyword]
                    }
                } else if p.token_idx(Some(0)).map(|t| t.value.to_lowercase() == "show").unwrap_or(false) {
                    return vec![SuggestType::Show]
                }
                let tables = extract_tables(full_text, &self.parser);
                vec![SuggestType::Column(tables)]
            },
            "set" | "order by" | "distinct" => {
                let tables = extract_tables(full_text, &self.parser);
                vec![SuggestType::Column(tables)]
            },
            "as" => vec![], // suggest nothing for an alias
            "show" => vec![SuggestType::Show],
            "to" => {
                let p = self.parser.parse_no_grouping(text_before_cursor);
                let first = &p[0].value.to_lowercase(); 
                if first == "change" {  vec![SuggestType::Change] } else { vec![SuggestType::User] }
            },
            "user" | "for" => vec![SuggestType::User],
            "select" | "where" | "having" => {
                let parent = identifier.map(|id| id.get_parent_name()).flatten();
                let tables = extract_tables(full_text, &self.parser);
                let mut suggestions = vec![];
                if let Some(p) = parent {
                    let tables = tables.into_iter().filter(|t| t.identifies(p)).collect::<Vec<_>>();
                    let s = vec![
                        SuggestType::Column(tables),
                        SuggestType::Table(p.to_string()),
                        SuggestType::View(p.to_string()),
                        SuggestType::Function(p.to_string()),
                    ];
                    suggestions.extend(s);
                } else {
                    let alias = tables.iter().map(|t| t.alias.clone().unwrap_or(t.table.clone())).collect::<Vec<_>>();
                    let s = vec![
                        SuggestType::Column(tables),
                        SuggestType::Function("".to_string()),
                        SuggestType::Alias(alias),
                        SuggestType::Keyword,
                    ];
                    suggestions.extend(s);
                }
                suggestions
            },
            v if v.ends_with("join") && token.map(|t| t.is_keyword()).unwrap_or(false) => suggest_schema(identifier, &token_v),
            "copy" | "from" | "update" | "into" | "describe" | "truncate" | "desc" | "explain" => {
                suggest_schema(identifier, &token_v)
            },
            // ALTER TABLE <tablname>
            "table" | "view" | "function" => {
                let parent = identifier.map(|id| id.get_parent_name()).flatten();
                let mut suggest_types = vec![];
                let mut schema_name = "".to_string();
                if let Some(schema) = parent { schema_name = schema.to_string(); } else {
                    suggest_types.push(SuggestType::Schema(None)); }
                let suggest_type = match token_v.as_ref() {
                    "table" => SuggestType::Table(schema_name),
                    "view" => SuggestType::View(schema_name),
                    "function" => SuggestType::Function(schema_name),
                    _ => unreachable!()
                };
                suggest_types.push(suggest_type);
                suggest_types
            }
            "on" => {
                let tables = extract_tables(full_text, &self.parser);
                let parent = identifier.map(|i| i.get_parent_name()).flatten();
                if parent.is_some() {
                    //  "ON parent.<suggestion>"
                    let parent = parent.unwrap();
                    let tables = tables.into_iter().filter(|t| t.identifies(parent)).collect::<Vec<_>>();
                    vec![
                        SuggestType::Column(tables),
                        SuggestType::Table(parent.to_string()),
                        SuggestType::View(parent.to_string()),
                        SuggestType::Function(parent.to_string()),
                    ]
                } else {
                    // ON <suggestion>
                    let aliases = tables.iter().map(|t| t.alias.as_ref().unwrap_or(&t.table).clone()).collect::<Vec<_>>();
                    if aliases.len() < 1 {
                        vec![SuggestType::Alias(aliases), SuggestType::Table("".to_string())]
                    } else {
                        vec![SuggestType::Alias(aliases)]
                    }
                }
            }
            "use" | "database" | "template" | "connect" => vec![SuggestType::Database],
            "tableformat" => vec![SuggestType::TableFormat],
            v if v.ends_with(",") || is_operand(v) || ["=", "and", "or"].contains(&v) => {
                let (prev_keyword, text_before_cursor) = find_prev_keyword(text_before_cursor, &self.parser);
                if let Some(prev_keyword) = prev_keyword {
                    self.suggest_based_on_last_token(Some(&prev_keyword), None, &text_before_cursor, full_text, identifier)
                } else {
                    vec![]
                }
            },
            _ => vec![SuggestType::Keyword, SuggestType::Special]
        }
    }

}

fn suggest_schema(identifier: Option<&Token>, token_v: &str) -> Vec<SuggestType> {
    let schema = identifier.map(|i| i.get_parent_name()).flatten();
    let mut suggest = vec![SuggestType::Table(schema.unwrap_or("").to_string())];
    if schema.is_none() {
        suggest = vec![SuggestType::Schema(None), suggest[0].clone()];
    }
    if token_v != "truncate" {
        suggest.push(SuggestType::View(schema.unwrap_or("").to_string()));
    }
    suggest
}

fn is_operand(op: &str) -> bool {
    match op {
        "+" | "-" | "*" | "/" => true,
        _ => false,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_type3() {
        let suggest = Suggest::default();
        let full_text = "select id from ";
        let suggestions = suggest.suggest_type(full_text, full_text);
        println!("{:?}", suggestions);
    }

    #[test]
    fn test_suggest_database() {
        let suggest = Suggest::default();
        let full_text = "use ";
        let suggestions = suggest.suggest_type(full_text, full_text);
        assert_eq!(suggestions[0], SuggestType::Database);
    }

    #[test]
    fn test_extract_tables() {
        let p = Parser::default();
        let sql = "select id from logs order by ";
        let tables = extract_tables(sql, &p);
        println!("{:?}", tables);
    }
}