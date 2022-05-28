use super::{TokenListFilter, next_token_align};
use crate::lexer::{Token, TokenList};
use crate::tokens::TokenType;

pub struct AlignedIndentFilter {
    pub n: String,
    pub offset: usize,
    pub indent: usize,
    pub chr: String,
    prev_sql: String,
    max_kwd_len: usize,
}

impl TokenListFilter for AlignedIndentFilter {

    fn process(&mut self, token_list: &mut TokenList) {
        self.process_default(token_list);
    }
}


impl AlignedIndentFilter {

    pub fn new(chr: &str, n: &str) -> Self {
        Self {
            n: n.to_string(),
            offset: 0,
            indent: 0,
            chr: chr.to_string(),
            max_kwd_len: 6,
            prev_sql: "".to_string(),
        }
    }

    fn nl(&self, offset: isize) -> Token {
        let indent = self.indent * (2 + self.max_kwd_len);
        let i = (self.max_kwd_len + indent + self.offset) as isize + offset;
        let i = if i > 0 { i as usize } else { 0 };
        let white = format!("{}{}", self.n, self.chr.repeat(i));
        Token::new(TokenType::Whitespace, &white)
    }

    fn split_kwds(&self, token_list: &mut TokenList) {
        let mut tidx = next_token_align(token_list, 0);
        while let Some(idx) = tidx {
            let token = token_list.token_idx(Some(idx)).unwrap();
            let token_indent = if token.is_keyword() && 
                (token.normalized.ends_with("JOIN") ||  token.normalized.ends_with("BY")) {
                token.normalized.split_whitespace().next().map(|s| s.len()).unwrap() as isize
            } else {
                0 - token.value.len() as isize
            };
            let nl = self.nl(token_indent);
            token_list.insert_before(idx, nl);
            tidx = next_token_align(token_list, idx+2)
        }
    }

    fn process_internal(&mut self, token_list: &mut TokenList, token_type: &TokenType) {
        match token_type {
            TokenType::IdentifierList => self.process_identifierlist(token_list),
            TokenType::Parenthesis => self.process_parenthesis(token_list),
            _ => self.process_default(token_list),
        }
    }

    fn process_parenthesis(&mut self, token_list: &mut TokenList) {
        let patterns = (TokenType::KeywordDML, vec!["SELECT"]);
        let tidx = token_list.token_next_by(&vec![], Some(&patterns), 0);
        if tidx.is_some() {
            self.indent += 1;
            token_list.insert_newline_after(0, self.nl(-6), true);
            self.process_default(token_list);
            self.indent -= 1;
            token_list.insert_newline_before(token_list.len()-1, self.nl(1));
        }
    }

    fn process_identifierlist(&mut self, token_list: &mut TokenList) {
        let identifiers = token_list.get_identifiers();
        let mut insert_count = 0;
        for identifier in identifiers.iter().skip(1) {
            if !token_list.insert_newline_before(identifier+insert_count,self.nl(1)) {
                insert_count += 1;
            }
        }
        self.process_default(token_list);
    }

    fn process_default(&mut self, token_list: &mut TokenList) {
        self.split_kwds(token_list);
        // prev
        for token in token_list.tokens.iter_mut() {
            if token.is_group() {
                // update offset
                let prev_sql = self.prev_sql.trim_end().to_lowercase();
                let offset = if prev_sql.ends_with("order by") || prev_sql.ends_with("group by") { 3 } else { 0 };
                self.offset += offset;
                self.process_internal(&mut token.children, &token.typ);
                token.update_value();
                self.offset -= offset;
            } else {
                self.prev_sql.push_str(&token.value)
            }
        }
    }
}