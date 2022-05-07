use super::{last_word, find_prev_keyword, extract_tables};
use sqlparse::{ parse, Token, TokenType, TokenList };

pub enum SuggestType {
    Keyword,
    Special,
    Table(String), // Option<String>
    Schema(Vec<String>),
    Column(Vec<SuggestTable>),
    View(Vec<String>),
    Function(Vec<String>),
    Alias(Vec<String>),
    Show,
}

#[derive(Debug)]
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

// TODO: support multiple statement
pub fn suggest_type(full_text: &str, text_before_cursor: &str) -> Vec<SuggestType> {

    let word_before_cursor = last_word(text_before_cursor, "many_punctuations");
    let mut identifier: Option<&Token> = None;
    let p = &parse(word_before_cursor)[0];
    
    let parsed: Vec<Token> =  if !word_before_cursor.is_empty() {
        if word_before_cursor.ends_with("(") {
            parse(text_before_cursor)
        } else {
            let l = text_before_cursor.len() - word_before_cursor.len();
            if p.children.len() > 0 && p.children.token_idx(Some(0)).map(|t| t.typ == TokenType::Identifier).unwrap_or(false) {
                identifier = p.children.token_idx(Some(0))
            }
            parse(&text_before_cursor[..l])
        }
    } else {
        parse(text_before_cursor)
    };

    let statement = TokenList::new(parsed);

    let last_token_idx = statement.token_prev(statement.len());
    let last_token = statement.token_idx(last_token_idx);

    suggest_based_on_last_token(last_token, text_before_cursor, full_text, identifier)
}

pub fn suggest_based_on_last_token(token: Option<&Token>, text_before_cursor: &str, full_text: &str, identifier: Option<&Token>) -> Vec<SuggestType> {
    if token.is_none() {
        return vec![SuggestType::Keyword, SuggestType::Special]
    }
    let token = token.unwrap();
    let mut token_v = "".to_string();
    if token.typ == TokenType::Comparison {
        let t = token.children.token_idx(Some(token.children.len()-1));
        token_v = t.unwrap().value.to_lowercase();
    } else if token.typ == TokenType::Where {
        let (prev_keyword, text_before_cursor) = find_prev_keyword(text_before_cursor);
        if !prev_keyword.as_ref().map(|t| t.typ == TokenType::Where).unwrap_or(false) {
            return suggest_based_on_last_token(prev_keyword.as_ref(), &text_before_cursor, full_text, identifier);
        }
    } else {
        token_v = token.value.to_lowercase();
    }
    match token_v.as_ref() {
        "set" | "order by" | "distinct" => vec![SuggestType::Keyword, SuggestType::Special],
        "select" | "where" | "having" => {
            let parent = identifier.map(|id| id.get_parent_name()).flatten();
            let tables = extract_tables(full_text);
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
        }
        _ => vec![SuggestType::Keyword, SuggestType::Special]
    }
}