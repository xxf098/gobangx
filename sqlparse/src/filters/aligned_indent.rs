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
                token.normalized.split_whitespace().next().map(|s| s.len()).unwrap()
            } else {
                token.value.len()
            };
            let nl = self.nl(0 - token_indent as isize);
            let forward = if token_list.insert_newline_before(idx, nl) { 1 } else { 2 };
            tidx = next_token_align(token_list, idx+forward)
        }
        // remove the last space
        if token_list.tokens[token_list.len()-1].is_whitespace() {
            token_list.tokens.remove(token_list.len()-1);
        }
    }

    fn process_internal(&mut self, token_list: &mut TokenList, token_type: &TokenType) {
        match token_type {
            TokenType::IdentifierList => self.process_identifierlist(token_list),
            TokenType::Parenthesis => self.process_parenthesis(token_list),
            TokenType::Case => self.process_case(token_list),
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

    fn process_case(&mut self, token_list: &mut TokenList) {
        let offset_ = 10; // len('case ') + len('when ')
        let mut cases = token_list.get_case(true);

        let pattern = (TokenType::Keyword, vec!["END"]);
        if let Some(end_idx) = token_list.token_next_by(&vec![], Some(&pattern), 0) {
            cases.push((vec![], vec![end_idx]));
        }

        let mut condition_width = cases.iter()
                .map(|c| c.0.iter().map(|idx| token_list.tokens[*idx].value.as_str()).collect::<Vec<_>>().join(" ").len());
        let first_cond_width = condition_width.next().unwrap_or(0);
        let max_cond_width = condition_width.max().unwrap_or(0).max(first_cond_width);
        
        let mut insert_count = 0;
        for (idx, (cond, value)) in cases.iter().enumerate() {
            let token_idx = if cond.len() > 0 {cond[0]} else {value[0]};
            if idx > 0 {
                let token_len = token_list.tokens[token_idx].value.len();
                let offset = offset_ as isize - token_len as isize;
                if !token_list.insert_newline_before(token_idx, self.nl(offset)) {
                    insert_count += 1;
                }
            }
            if cond.len() > 0 {
                let n = max_cond_width.saturating_sub(first_cond_width);
                let white = self.chr.repeat(n);
                let ws = Token::new(TokenType::Whitespace, &white);
                let last = cond.last().unwrap() + insert_count;
                if !token_list.insert_newline_after(last, ws, true) {
                    insert_count += 1;
                }
            }
        }
    }

    fn process_default(&mut self, token_list: &mut TokenList) {
        self.split_kwds(token_list);
        // prev
        let mut remove_indexes = vec![]; // handle newline in first position
        for (i, token) in token_list.tokens.iter_mut().enumerate() {
            if token.is_group() {
                // update offset
                let prev_sql = self.prev_sql.trim_end().to_lowercase();
                let offset = if prev_sql.ends_with("order by") || prev_sql.ends_with("group by") { 3 } else { 0 };
                self.offset += offset;
                self.process_internal(&mut token.children, &token.typ);
                token.update_value();
                if token.value.starts_with("\n") && i > 0 {
                    remove_indexes.push(i-1);
                }
                self.offset -= offset;
            } else {
                self.prev_sql.push_str(&token.value)
            }
        }
        remove_indexes.iter().enumerate().for_each(|(i, idx)| {token_list.tokens.remove(idx-i);});
    }
}