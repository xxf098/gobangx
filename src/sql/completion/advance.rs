use std::convert::TryFrom;
use super::Completion;
use crate::config::{DatabaseType};
use crate::sql::token::{
    tokenizer::{Tokenizer, Token},
    TokenType,
};

pub enum Type {
    Keyword,
    Column,
    Table,
}


pub struct SuggestType {
    typ: Type,
    data: Vec<String>,
}

impl Default for SuggestType {
    fn default() -> Self {
        Self { typ: Type::Keyword, data: vec![] }
    }
}

// TODO: mysql pg
pub struct Advance {
    db_type: DatabaseType,
    tokens: Vec<Token>,
}


impl Advance {

    // full_text is used for extract table name
    fn suggest_type(&mut self, full_text: &str, text_before_cursor: &str) -> anyhow::Result<SuggestType>{
        // FIXME: 
        let t = Tokenizer::try_from(self.db_type.clone())?;
        self.tokens = t.tokenize(text_before_cursor);
        let last_token = self.tokens.last();
        Ok(self.suggest_based_on_last_token(last_token, full_text, text_before_cursor))
    }

    fn suggest_based_on_last_token(&self, last_token: Option<&Token>, full_text: &str, text_before_cursor: &str) -> SuggestType {
        if last_token.is_none() {
            return SuggestType::default()
        }
        let token_v = &last_token.unwrap().value.to_uppercase();
        match token_v.as_ref() {
            "SELECT" | "WHERE" | "HAVING" => {
                
                SuggestType::default()
            },
            _ => SuggestType::default()
        }
    }

    fn extract_tables(&self, sql: &str) -> Vec<String>{
        if self.tokens.len() < 1 {
            return vec![]
        }


        vec![]
    }

    fn extract_from_part(&self, stop_at_punctuation: bool) -> Vec<String> {
        let mut tbl_prefix_seen = false;
        let mut tables = vec![];
        for item in &self.tokens {
            if tbl_prefix_seen {
                if stop_at_punctuation && item.typ == TokenType::OpenParen {
                    return vec![]
                } else if item.typ == TokenType::Reserved || item.typ == TokenType::ReservedTopLevel {
                    let value = item.value.to_uppercase();
                    if value == "ON" {
                        continue
                    }
                    if value == "FROM" || value == "JOIN" {
                        return tables
                    }
                } else {
                    tables.push(item.value.clone());
                }
            } else if item.typ == TokenType::Reserved || item.typ == TokenType::ReservedTopLevel {
                let value = item.value.to_uppercase();
                if value == "COPY" || value == "FROM" || value == "INTO" || value == "UPDATE" || value == "TABLE" || value == "JOIN" {
                    tbl_prefix_seen = true;
                }
            }
        };
        tables
    }
}

impl Completion for Advance {
    fn new(db_type: DatabaseType, candidates: Vec<String>) -> Self {
        Advance{ db_type, tokens: vec![]}
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
    fn test_extract_from_part() {
        let mut adv = Advance::new(DatabaseType::Postgres, vec![]);
        let sql = "select * from users";
        adv.suggest_type(sql, sql).unwrap();
        let tables = adv.extract_from_part(false);
        assert_eq!(tables, vec!["users"]);
    }
}