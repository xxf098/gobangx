use super::TokenListFilter;
use crate::lexer::{Token, TokenList};
use crate::tokens::TokenType;

const SPLIT_WORDS: [&str; 14] = ["FROM", "STRAIGHT_JOIN", "JOIN", "AND", "OR", "GROUP BY", 
    "ORDER BY", "UNION", "VALUES", "SET", "BETWEEN", "EXCEPT", "HAVING", "LIMIT"];

pub struct ReindentFilter {
    n: String, // newline
    width: usize,
    chr: String, // indent space character
    indent: usize,
    offset: usize,
    wrap_after: usize,
    comma_first: bool,
    indent_columns: bool,

}

impl TokenListFilter for ReindentFilter {

    fn process(&mut self, token_list: &mut TokenList) {
        self.process_default(token_list);
    }
}


impl ReindentFilter {

    pub fn new(width: usize, chr: &str, wrap_after: usize, n: &str, 
        comma_first: bool, indent_after_first: bool, indent_columns: bool) -> Self {
        Self {
            n: n.to_string(),
            width,
            chr: chr.to_string(),
            indent: if indent_after_first { 1 } else { 0},
            offset: 0,
            wrap_after,
            comma_first,
            indent_columns,
        }
    }

    fn leading_ws(&self) -> usize {
        self.offset + self.indent * self.width
    }

    fn nl(&self, offset: usize) -> Token {
        let white = format!("{}{}", self.n, self.chr.repeat(self.leading_ws()+offset));
        Token::new(TokenType::Whitespace, &white)
    }

    fn next_token(&self, token_list: &TokenList, idx: usize) -> Option<usize> {
        let patterns = (TokenType::Keyword, SPLIT_WORDS.to_vec());
        let mut tidx = token_list.token_next_by(&vec![], Some(&patterns), idx);
        let token = token_list.token_idx(tidx);
        if token.map(|t| t.normalized == "BETWEEN").unwrap_or(false) {
            tidx = self.next_token(token_list, tidx.unwrap()+1);
            let token = token_list.token_idx(tidx);
            if token.map(|t| t.normalized == "AND").unwrap_or(false) {
                tidx = self.next_token(token_list, tidx.unwrap()+1);
            } 
        }
        tidx
    }

    fn split_kwds(&self, token_list: &mut TokenList) {
        let mut tidx = self.next_token(token_list, 0);
        while let Some(mut idx) = tidx {
            let pidx = token_list.token_prev(idx, false);
            let prev = token_list.token_idx(pidx);
            let is_newline = prev.map(|t| t.value.ends_with("\n") || t.value.ends_with("\r")).unwrap_or(false);
            if prev.map(|t| t.is_whitespace()).unwrap_or(false) {
                token_list.tokens.remove(pidx.unwrap());
                idx -= 1;
            }
            if !is_newline {
                token_list.insert_before(idx, self.nl(0));
                idx += 1;
            }
            tidx = self.next_token(token_list, idx+1)
        }
    }

    fn split_statements(&self, token_list: &mut TokenList) {
        let ttypes = vec![TokenType::KeywordDML, TokenType::KeywordDDL];
        let mut tidx = token_list.token_next_by(&ttypes, None, 0);
        while let Some(mut idx) = tidx {
            let pidx = token_list.token_prev(idx, false);
            let prev = token_list.token_idx(pidx);
            if prev.map(|t| t.is_whitespace()).unwrap_or(false) {
                token_list.tokens.remove(pidx.unwrap());
                idx -= 1;
            }
            if pidx.is_some() {
                token_list.insert_before(idx, self.nl(0));
                idx += 1;
            }
            tidx = token_list.token_next_by(&ttypes, None, idx+1) 
        }
    }


    fn process_internal(&mut self, token_list: &mut TokenList, token_type: &TokenType) {
        match token_type {
            TokenType::Where => self.process_where(token_list),
            _ => self.process_default(token_list),
        }
    }

    fn process_where(&mut self, token_list: &mut TokenList) {
        let patterns = (TokenType::Keyword, vec!["WHERE"]);
        let tidx = token_list.token_next_by(&vec![], Some(&patterns), 0);
        if let Some(idx) = tidx {
            token_list.insert_before(idx, self.nl(0));
            self.indent += 1;
            self.process_default(token_list);
            self.indent -= 1;
        }
    }

    fn process_default(&mut self, token_list: &mut TokenList) {
        self.split_statements(token_list);
        self.split_kwds(token_list);
        for token in token_list.tokens.iter_mut() {
            if token.is_group() {
                self.process_internal(&mut token.children, &token.typ);
            }
        }
    }
}


