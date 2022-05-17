use crate::lexer::{Token, tokenize_internal};
use crate::keywords::{RegexToken, sql_regex};
use crate::filters::Filter;

// 'a
pub struct FilterStack {
    regs: Vec<RegexToken>,
    preprocess: Vec<Box<dyn Filter>>,
    stmtprocess: Vec<Box<dyn Filter>>,
    postprocess: Vec<Box<dyn Filter>>,
}


impl FilterStack {

    pub fn new() -> Self {
        Self { 
            regs: sql_regex(), 
            preprocess: vec![],
            stmtprocess: vec![],
            postprocess: vec![], 
        }
    }

    // TODO: support more than one sql
    pub fn run(&self, sql: &str, grouping: bool) -> Vec<Token> {
        let mut tokens = tokenize_internal(sql, &self.regs);
        if grouping {
            tokens = super::grouping::group(tokens);
        }
        tokens
    }

    // format sql
    pub fn format(&self, sql: &str) -> Vec<Token> {
        let mut tokens = tokenize_internal(sql, &self.regs);
        for token in tokens.iter_mut() {
            self.preprocess.iter().for_each(|filter| filter.process(token));
            self.stmtprocess.iter().for_each(|filter| filter.process(token));
            self.postprocess.iter().for_each(|filter| filter.process(token));
        }
        tokens
    }
}