use std::convert::TryFrom;
use super::Completion;
use crate::config::{DatabaseType};
use crate::sql::token::{tokenizer::{Tokenizer, Token} };

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

}

impl Advance {

    // full_text is used for extract table name
    fn suggest_type(&self, full_text: &String, text_before_cursor: &String) -> anyhow::Result<SuggestType>{
        // FIXME: 
        let t = Tokenizer::try_from(DatabaseType::Postgres)?;
        let tokens = t.tokenize(text_before_cursor);
        let last_token = tokens.last();
        Ok(self.suggest_based_on_last_token(last_token, full_text, text_before_cursor))
    }

    fn suggest_based_on_last_token(&self, last_token: Option<&Token>, full_text: &String, text_before_cursor: &String ) -> SuggestType {
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


}

impl Completion for Advance {
    fn new(candidates: Vec<String>) -> Self {
        Advance{}
    }

    fn complete(&self, full_text: String, word: &String) -> Vec<&String> {
        vec![]
    }

    fn update_candidates(&mut self, candidates: &[String]) {

    }
}
