use super::StmtFilter;
use crate::lexer::{Token, TokenList};

pub struct StripWhitespaceFilter { }

impl StripWhitespaceFilter {

    fn stripws(tokens: &mut Vec<Token>) {
        StripWhitespaceFilter::stripws_default(tokens);
    }

    fn stripws_default(tokens: &mut Vec<Token>) {
        let mut last_was_ws = false;
        let mut is_first_char = true;
        for token in tokens.iter_mut() {
            if token.is_whitespace() {
                if last_was_ws || is_first_char { token.value = "".to_string() };
            }
            last_was_ws = token.is_whitespace();
            is_first_char = false;
        }
    }

}

impl StmtFilter for StripWhitespaceFilter {

    // strip which
    fn process(&self, tokens: &mut Vec<Token>, depth: usize) {
        for token in tokens.iter_mut() {
            if token.is_group() {
                self.process(&mut token.children.tokens, depth+1);
                token.update_value();
            }
        }
        Self::stripws(tokens);
    }
}

