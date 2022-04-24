use crate::sql::parse::lexer::{Token, tokenize};

pub struct FilterStack {
    grouping: bool
}


impl FilterStack {

    pub fn new(grouping: bool) -> Self {
        Self { grouping }
    }

    fn enable_grouping(&mut self) {
        self.grouping = true
    }

    // TODO: support more than one sql
    pub fn run(&self, sql: &str) -> Vec<Token> {
        let mut tokens = tokenize(sql);
        if self.grouping {
            tokens = super::grouping::group(tokens);
        }
        tokens
    }
}