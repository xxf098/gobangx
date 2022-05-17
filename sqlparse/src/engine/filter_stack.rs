use crate::lexer::{Token, tokenize_internal};
use crate::keywords::{RegexToken, sql_regex};
use crate::filters::Filter;

// 'a
pub struct FilterStack {
    regs: Vec<RegexToken>,
    preprocess: Vec<Box<dyn Filter>>,
}


impl FilterStack {

    pub fn new() -> Self {
        Self { regs: sql_regex(), preprocess: vec![] }
    }

    // TODO: support more than one sql
    pub fn run(&self, sql: &str, grouping: bool) -> Vec<Token> {
        let mut tokens = tokenize_internal(sql, &self.regs);
        if grouping {
            tokens = super::grouping::group(tokens);
        }
        tokens
    }
}