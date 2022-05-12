use super::{last_word, find_prev_keyword, extract_tables};
use sqlparse::{ Token, TokenType, TokenList, Parser };

#[derive(Debug, Clone, PartialEq)]
pub enum SuggestType {
    Keyword,
    Special,
    Database,
    Schema(Option<String>), // database name
    Table(String), // schema name
    View(String),
    Column(Vec<SuggestTable>),
    Function(Vec<String>),
    Alias(Vec<String>),
    // Show,
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
}

#[derive(Default)]
pub struct Suggest {
    parser: Parser
}

impl Suggest {
    
    // TODO: support multiple statement
    pub fn suggest_type(&self, full_text: &str, text_before_cursor: &str) -> Vec<SuggestType> {
        let word_before_cursor = last_word(text_before_cursor, "many_punctuations");
        let mut identifier: Option<&Token> = None;
        let mut tokens = vec![];
        
        let parsed: Vec<Token> =  if !word_before_cursor.is_empty() {
            if word_before_cursor.ends_with("(") {
                self.parser.parse(text_before_cursor)
            } else {
                tokens = self.parser.parse(word_before_cursor);
                let p = &tokens[0];
                let l = text_before_cursor.len() - word_before_cursor.len();
                if p.children.len() > 0 && p.children.token_idx(Some(0)).map(|t| t.typ == TokenType::Identifier).unwrap_or(false) {
                    identifier = p.children.token_idx(Some(0))
                }
                self.parser.parse(&text_before_cursor[..l])
            }
        } else {
            self.parser.parse(text_before_cursor)
        };

    
        let statement = TokenList::new(parsed);
    
        let last_token_idx = statement.token_prev(statement.len());
        let last_token = statement.token_idx(last_token_idx);
   
    
        let suggests = self.suggest_based_on_last_token(last_token, text_before_cursor, full_text, identifier);
        suggests
    }

    pub fn suggest_based_on_last_token(&self, token: Option<&Token>, text_before_cursor: &str, full_text: &str, identifier: Option<&Token>) -> Vec<SuggestType> {
        if token.is_none() {
            return vec![SuggestType::Keyword, SuggestType::Special]
        }
        let token = token.unwrap();
        let mut token_v = "".to_string();
        if token.typ == TokenType::Comparison {
            let t = token.children.token_idx(Some(token.children.len()-1));
            token_v = t.unwrap().value.to_lowercase();
        } else if token.typ == TokenType::Where {
            let (prev_keyword, text_before_cursor) = find_prev_keyword(text_before_cursor, &self.parser);
            if !prev_keyword.as_ref().map(|t| t.typ == TokenType::Where).unwrap_or(false) {
                return self.suggest_based_on_last_token(prev_keyword.as_ref(), &text_before_cursor, full_text, identifier);
            }
        } else {
            token_v = token.value.to_lowercase();
        }
        match token_v.as_ref() {
            "set" | "order by" | "distinct" => vec![SuggestType::Keyword, SuggestType::Special],
            "select" | "where" | "having" => {
                let parent = identifier.map(|id| id.get_parent_name()).flatten();
                let tables = extract_tables(full_text, &self.parser);
                let mut suggestions = vec![];
                if let Some(_p) = parent {
                    unimplemented!()
                } else {
                    let alias = tables.iter().map(|t| t.alias.clone().unwrap_or(t.table.clone())).collect::<Vec<_>>();
                    let s = vec![
                        SuggestType::Column(tables),
                        SuggestType::Function(vec![]),
                        SuggestType::Alias(alias),
                        SuggestType::Keyword,
                    ];
                    suggestions.extend(s);
                }
                suggestions
            },
            "as" => vec![], // suggest nothing for an alias
            v if v.ends_with("join") && token.is_keyword() => {
                suggest_schema(identifier, &token_v)
            },
            "copy" | "from" | "update" | "into" | "describe" | "truncate" | "desc" | "explain" => {
                suggest_schema(identifier, &token_v)
            },
            "use" | "database" | "template" | "connect" => {
                vec![SuggestType::Database]
            }
            v if v.ends_with(",") || is_operand(v) || ["=", "and", "or"].contains(&v) => {
                let (prev_keyword, text_before_cursor) = find_prev_keyword(text_before_cursor, &self.parser);
                if let Some(prev_keyword) = prev_keyword {
                    self.suggest_based_on_last_token(Some(&prev_keyword), &text_before_cursor, &text_before_cursor, identifier)
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
}