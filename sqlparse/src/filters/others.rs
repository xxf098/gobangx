use super::StmtFilter;
use crate::lexer::{Token};
use crate::tokens::TokenType;

pub struct StripWhitespaceFilter { }

impl StripWhitespaceFilter {

    fn stripws(tokens: &mut Vec<Token>) {
        StripWhitespaceFilter::stripws_default(tokens);
        StripWhitespaceFilter::stripws_newline(tokens);
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

    // remove whitespace after newline
    fn stripws_newline(tokens: &mut Vec<Token>) {
        let mut idx = 0;
        while idx < tokens.len() {
            let token = &tokens[idx];
            if token.typ != TokenType::Newline {
                idx += 1;
                continue
            }
            let next_idx = idx+1;
            while next_idx < tokens.len() {
                let token_next = &tokens[next_idx];
                if !token_next.is_whitespace() {
                    break
                }
                tokens.remove(next_idx);
            }
            idx += 1;
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

    fn process(&self, tokens: &mut Vec<Token>) {
        for token in tokens.iter_mut() {
            if token.is_group() {
                Self::stripws_parenthesis(token);
                self.process(&mut token.children.tokens);
                token.update_value();
            }
        }
        Self::stripws(tokens);
    }
}


// trim space before newline
pub struct StripBeforeNewline{}

impl StmtFilter for StripBeforeNewline {

    fn process(&self, tokens: &mut Vec<Token>) {
        let mut remove_indexes = vec![];
        let mut is_before_white = false;
        for (i, token) in tokens.iter_mut().enumerate() {         
            if token.is_group() {
                self.process(&mut token.children.tokens);
            }
            if is_before_white && token.value.starts_with("\n") && i > 0 {
                remove_indexes.push(i-1)
            }
            is_before_white = token.is_whitespace();
        }
        remove_indexes.iter().enumerate().for_each(|(i, idx)| {tokens.remove(idx-i);});
    }

} 
