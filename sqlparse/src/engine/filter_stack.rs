use crate::lexer::{Token, tokenize_internal};
use crate::keywords::{RegexToken, sql_regex};
use crate::filters::Filter;

// 'a
pub struct FilterStack {
    regs: Vec<RegexToken>,
    pub preprocess: Vec<Box<dyn Filter>>,
    pub stmtprocess: Vec<Box<dyn Filter>>,
    pub postprocess: Vec<Box<dyn Filter>>,
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
    pub fn format(&self, sql: &str, grouping: bool) -> Vec<Token> {
        let mut tokens = tokenize_internal(sql, &self.regs);
        if grouping {
            for token in tokens.iter_mut() {
                self.preprocess.iter().for_each(|filter| filter.process(token, 0));
            }
            tokens = super::grouping::group(tokens);
            for token in tokens.iter_mut() {
                self.stmtprocess.iter().for_each(|filter| filter.process(token, 0));
                self.postprocess.iter().for_each(|filter| filter.process(token, 0));
            }
        } else {
            for token in tokens.iter_mut() {
                self.preprocess.iter().for_each(|filter| filter.process(token, 0));
                self.stmtprocess.iter().for_each(|filter| filter.process(token, 0));
                self.postprocess.iter().for_each(|filter| filter.process(token, 0));
            }
        }
        tokens
    }
}