use crate::lexer::{Token, TokenList, tokenize_internal};
use crate::keywords::{RegexToken, sql_regex};
use crate::filters::{Filter, StmtFilter, TokenListFilter};

// 'a
pub struct FilterStack {
    regs: Vec<RegexToken>,
    pub preprocess: Vec<Box<dyn Filter>>,
    pub stmtprocess: Vec<Box<dyn StmtFilter>>,
    pub tlistprocess: Vec<Box<dyn TokenListFilter>>,
    pub postprocess: Vec<Box<dyn StmtFilter>>,
}


impl FilterStack {

    pub fn new() -> Self {
        Self { 
            regs: sql_regex(), 
            preprocess: vec![],
            stmtprocess: vec![],
            postprocess: vec![],
            tlistprocess: vec![],
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
    pub fn format(&mut self, sql: &str, grouping: bool) -> Vec<Token> {
        let mut tokens = tokenize_internal(sql, &self.regs);
        for token in tokens.iter_mut() {
            self.preprocess.iter().for_each(|filter| filter.process(token));
        }
        if grouping {
            tokens = super::grouping::group(tokens);
        }
        self.stmtprocess.iter().for_each(|filter| filter.process(&mut tokens));
        let mut token_list = TokenList{ tokens: tokens };
        self.tlistprocess.iter_mut().for_each(|filter| filter.process(&mut token_list));
        tokens = std::mem::replace(&mut token_list.tokens, vec![]);
        self.postprocess.iter().for_each(|filter| filter.process(&mut tokens));
        tokens
    }
}