use crate::lexer::{Token, tokenize_internal};
use crate::keywords::{RegexToken, sql_regex};

pub struct FilterStack {
    regs: Vec<RegexToken>,
}


impl FilterStack {

    pub fn new() -> Self {
        Self { regs: sql_regex() }
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