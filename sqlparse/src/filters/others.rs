use super::StmtFilter;
use crate::lexer::{Token};
use crate::tokens::TokenType;

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
                token.value = if last_was_ws || is_first_char { "".to_string() } else { " ".to_string() };
            }
            last_was_ws = token.is_whitespace();
            is_first_char = false;
        }
    }

    fn stripws_parenthesis(token: &mut Token) {
        if token.typ != TokenType::Parenthesis {
            return
        }
        if token.children.token_idx(Some(1)).map(|t| t.is_whitespace()).unwrap_or(false) {
            token.children.tokens.remove(1);
        }
        let token_len = token.children.len();
        if token_len> 2 && token.children.token_idx(Some(token_len-2)).map(|t| t.is_whitespace()).unwrap_or(false) {
            token.children.tokens.remove(token_len-2);
        }
    }

}

impl StmtFilter for StripWhitespaceFilter {

    // strip which
    fn process(&self, tokens: &mut Vec<Token>, depth: usize) {
        for token in tokens.iter_mut() {
            if token.is_group() {
                Self::stripws_parenthesis(token);
                self.process(&mut token.children.tokens, depth+1);
                token.update_value();
            }
        }
        Self::stripws(tokens);
        // pop
    }
}

