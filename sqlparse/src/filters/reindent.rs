use super::TokenListFilter;
use crate::lexer::{Token, TokenList};
use crate::tokens::TokenType;

const SPLIT_WORDS: [&str; 12] = ["FROM", "AND", "OR", "GROUP BY", 
    "ORDER BY", "UNION", "VALUES", "SET", "BETWEEN", "EXCEPT", "HAVING", "LIMIT"];

pub struct ReindentFilter {
    n: String, // newline
    width: usize,
    chr: String, // indent space character
    indent: usize,
    offset: usize,
    prev_sql: String, // accumulate previous token to sql
    wrap_after: usize,
    comma_first: bool,
    indent_columns: bool,

}

impl TokenListFilter for ReindentFilter {

    fn process(&mut self, token_list: &mut TokenList) {
        self.process_default(token_list, true, 0);
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
            prev_sql: "".to_string(),
            wrap_after,
            comma_first,
            indent_columns,
        }
    }

    // fn flatten_up_to_token(&self, token_list: &TokenList, idx: usize) {
    //     unimplemented!()
    // }

    fn leading_ws(&self) -> usize {
        self.offset + self.indent * self.width
    }

    fn get_offset(&self, extra_str: &str) -> usize {
        let s = format!("{}{}", self.prev_sql, extra_str);
        let line = s.split('\n').last().unwrap_or("");
        // println!("line: {:?}", line);
        line.len().saturating_sub(self.chr.len()*self.leading_ws())
    }

    fn nl(&self, offset: usize) -> Token {
        let white = format!("{}{}", self.n, self.chr.repeat(self.leading_ws()+offset));
        Token::new(TokenType::Whitespace, &white)
    }

    fn next_token(&self, token_list: &TokenList, idx: usize) -> Option<usize> {
        let mut tidx = token_list.token_next_by_fn(|t| t.typ == TokenType::Keyword && 
            (SPLIT_WORDS.iter().find(|w| **w == t.normalized).is_some() || t.normalized.ends_with("STRAIGHT_JOIN") || t.normalized.ends_with("JOIN")), idx);
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
                // println!("{}", "nl split_kwds");
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
                // println!("{}", "nl split_statements");
                token_list.insert_before(idx, self.nl(0));
                idx += 1;
            }
            tidx = token_list.token_next_by(&ttypes, None, idx+1) 
        }
    }


    fn process_internal(&mut self, token_list: &mut TokenList, token_type: &TokenType, level: usize) {
        match token_type {
            TokenType::Where => self.process_where(token_list),
            TokenType::Parenthesis => self.process_parenthesis(token_list),
            TokenType::Values => self.process_values(token_list),
            TokenType::Case => self.process_case(token_list),
            _ => self.process_default(token_list, true, level),
        }
    }

    fn process_where(&mut self, token_list: &mut TokenList) {
        let patterns = (TokenType::Keyword, vec!["WHERE"]);
        let tidx = token_list.token_next_by(&vec![], Some(&patterns), 0);
        if let Some(idx) = tidx {
            token_list.insert_before(idx, self.nl(0));
            self.indent += 1;
            self.process_default(token_list, true, 1);
            self.indent -= 1;
        }
    }

    fn process_parenthesis(&mut self, token_list: &mut TokenList) {
        let patterns = (TokenType::Punctuation, vec!["("]);
        let pidx = token_list.token_next_by(&vec![], Some(&patterns), 0);
        if pidx.is_none() {
            return
        }
        let ttypes = vec![TokenType::KeywordDML, TokenType::KeywordDDL];
        let tidx = token_list.token_next_by(&ttypes, None, 0);

        let indent = if tidx.is_some() { 1 } else { 0 };
        self.indent += indent;
        let offset = if tidx.is_some() {
            let t = self.nl(0);
            let offset = self.get_offset(&t.value);
            token_list.insert_before(0, t);
            offset+1
        } else { self.get_offset("")+1 };
        self.offset += offset;
        self.process_default(token_list, tidx.is_none(), 1);
        self.offset -= offset;
        self.indent -= indent;
    }

    fn process_case(&mut self, token_list: &mut TokenList) {
        // println!("token_list: {}", token_list);
        let cases = token_list.get_case(false);
        let cond = &cases[0];
        let first = cond.0[0];
        {
            let offset = self.get_offset("");
            self.offset += offset;
            {
                let offset = self.get_offset("");
                self.offset += offset;
                for (cond, value) in cases.iter().skip(1) {
                    let token_idx = if cond.len() < 1 { value[0] } else { cond[0] };
                    token_list.insert_before(token_idx, self.nl(0));
                    {
                        let n = "WHEN ".len();
                        self.offset += n;
                        self.process_default(token_list, true, 1);
                        self.offset -= n;
                    }
                }
                let pattern = (TokenType::Keyword, vec!["END"]);
                let end_idx = token_list.token_next_by(&vec![], Some(&pattern), 0);
                if let Some(idx) = end_idx {
                    token_list.insert_before(idx, self.nl(0))
                }
                self.offset -= offset;
            }
            self.offset -= offset;
        }
    }

    fn process_values(&mut self, token_list: &mut TokenList) {
        token_list.insert_before(0, self.nl(0));
        let ttypes = vec![TokenType::Parenthesis];
        let mut tidx = token_list.token_next_by(&ttypes, None, 0);
        // let first_idx = tidx;
        while let Some(idx) = tidx {
            let patterns = (TokenType::Punctuation, vec![","]);
            let pidx = token_list.token_next_by(&vec![], Some(&patterns), idx);
            if let Some(idx1) = pidx {
                if self.comma_first {
                    let offset = self.get_offset("");
                    token_list.insert_before(idx1, self.nl(offset));
                } else {
                    let offset = self.get_offset("");
                    let nl = self.nl(offset);
                    token_list.insert_after(idx1, nl, true);
                }
            }
            tidx = token_list.token_next_by(&ttypes, None, idx+1); 
        }
    }

    fn process_default(&mut self, token_list: &mut TokenList, split: bool, level: usize) {
        if split { self.split_statements(token_list); }
        self.split_kwds(token_list);
        let mut remove_indexes = vec![];
        for (i, token) in token_list.tokens.iter_mut().enumerate() {
            if token.is_group() {
                self.process_internal(&mut token.children, &token.typ, 1);
                token.update_value();
                
                if token.value.starts_with("\n") && i > 0 {
                    remove_indexes.push(i-1);
                }
            }
            // top level only
            if level == 0 {
                self.prev_sql.push_str(&token.value);
            }
        }
        remove_indexes.iter().enumerate().for_each(|(i, idx)| {token_list.tokens.remove(idx-i);});
    }
}


