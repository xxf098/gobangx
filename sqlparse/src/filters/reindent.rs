use super::{TokenListFilter, next_token};
use crate::lexer::{Token, TokenList};
use crate::tokens::TokenType;

pub struct ReindentFilter {
    n: String, // newline
    width: usize,
    chr: String, // indent space character
    indent: usize,
    offset: usize,
    prev_sql: String, // accumulate previous token to sql
    // sub_prev_sql: String, // accumulate sub previous token to sql
    wrap_after: usize,
    comma_first: bool,
    indent_columns: bool,
    // parents_type: Option<TokenType>,
    last_func_len: usize,
}

impl TokenListFilter for ReindentFilter {

    fn process(&mut self, token_list: &mut TokenList) {
        self.process_default(token_list, true, vec![]);
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
            last_func_len: 0, 
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

    fn nl(&self, offset: isize) -> Token {
        let i = 0.max(self.leading_ws() as isize +offset) as usize;
        let white = format!("{}{}", self.n, self.chr.repeat(i));
        Token::new(TokenType::Whitespace, white)
    }

    // fn next_token(&self, token_list: &TokenList, idx: usize) -> Option<usize> {
    //     let mut tidx = token_list.token_next_by_fn(|t| t.typ == TokenType::Keyword && 
    //         (SPLIT_WORDS.iter().find(|w| **w == t.normalized).is_some() || t.normalized.ends_with("STRAIGHT_JOIN") || t.normalized.ends_with("JOIN")), idx);
    //     let token = token_list.token_idx(tidx);
    //     if token.map(|t| t.normalized == "BETWEEN").unwrap_or(false) {
    //         tidx = self.next_token(token_list, tidx.unwrap()+1);
    //         let token = token_list.token_idx(tidx);
    //         if token.map(|t| t.normalized == "AND").unwrap_or(false) {
    //             tidx = self.next_token(token_list, tidx.unwrap()+1);
    //         } 
    //     }
    //     tidx
    // }

    fn split_kwds(&self, token_list: &mut TokenList) {
        let mut tidx = next_token(token_list, 0);
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
            tidx = next_token(token_list, idx+1)
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


    fn process_internal(&mut self, token_list: &mut TokenList, token_type: &TokenType, parents: Vec<TokenType>) {
        match token_type {
            TokenType::Where => self.process_where(token_list, parents),
            TokenType::Parenthesis => self.process_parenthesis(token_list, parents),
            TokenType::Values => self.process_values(token_list),
            TokenType::Case => self.process_case(token_list, parents),
            TokenType::IdentifierList => self.process_identifierlist(token_list, parents),
            TokenType::Function => self.process_function(token_list, parents),
            _ => self.process_default(token_list, true, parents),
        }
    }

    fn process_where(&mut self, token_list: &mut TokenList, mut parents: Vec<TokenType>) {
        let patterns = (TokenType::Keyword, vec!["WHERE"]);
        let tidx = token_list.token_next_by(&vec![], Some(&patterns), 0);
        if let Some(idx) = tidx {
            token_list.insert_before(idx, self.nl(0));
            self.indent += 1;
            parents.push(TokenType::Where);
            self.process_default(token_list, true, parents);
            self.indent -= 1;
        }
    }

    fn process_parenthesis(&mut self, token_list: &mut TokenList, mut parents: Vec<TokenType>) {
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
        parents.push(TokenType::Parenthesis);
        self.process_default(token_list, tidx.is_none(), parents);
        self.offset -= offset;
        self.indent -= indent;
    }

    fn process_function(&mut self, token_list: &mut TokenList, mut parents: Vec<TokenType>) {
        self.last_func_len = token_list.tokens[0].value.len();
        parents.push(TokenType::Function);
        self.process_default(token_list, true, parents);
        self.last_func_len = 0;
    }

    fn process_identifierlist(&mut self, token_list: &mut TokenList, mut parents: Vec<TokenType>) {
        // println!("{}", token_list);
        let mut identifiers = token_list.get_identifiers();
        // println!("{:?}", identifiers);
        let num_offset = if self.indent_columns {
            if self.chr == "\n" { 1 } else { self.width }
        } else {
            if self.chr == "\t" { 1 } else {
                let first = identifiers.remove(0);
                let extra = token_list.tokens.iter().take(first).map(|t| t.value.as_str()).collect::<Vec<&str>>().join("");
                self.get_offset(&extra)
            }
        };
        if parents.iter().find(|t| **t == TokenType::Function || **t == TokenType::Values).is_none() {
            self.offset += num_offset;
            let mut position = 0;
            let mut insert_count = 0;
            for mut tidx in identifiers {
                tidx += insert_count;
                let token = token_list.token_idx(Some(tidx)).unwrap();
                // Add 1 for the "," separator
                position += token.value.len() + 1;
                if position + self.offset > self.wrap_after {
                    let mut adjust: isize = 0;
                    if self.comma_first {
                        adjust = -2;
                        let comma_idx = token_list.token_prev(tidx, true);
                        if comma_idx.is_none() {
                            continue
                        }
                        tidx = comma_idx.unwrap();
                    }
                    let is_space_removed = token_list.insert_newline_before(tidx, self.nl(adjust));
                    if !is_space_removed { insert_count += 1; }
                    if self.comma_first {
                        let count = if is_space_removed { 0 } else { 1 };
                        let ws_idx = token_list.token_next(tidx+count+1, false);
                        let ws = token_list.token_idx(ws_idx);
                        if ws.is_some() && ws.unwrap().typ != TokenType::Whitespace {
                            token_list.insert_after(tidx+count, Token::new(TokenType::Whitespace, " "), true);
                            insert_count += 1;
                        }
                    }
                    position = 0;
                }
            }
            self.offset -= num_offset;
        } else {
            let mut n = 0;
            while n < token_list.len() {
                let token = token_list.token_idx(Some(n)).unwrap();
                if token.value != "," {
                    n += 1;
                    continue
                }
                let next_id = token_list.token_next(0, false);
                let next_ws = token_list.token_idx(next_id);
                if next_ws.map(|t| t.is_whitespace()).unwrap_or(false) {
                    token_list.insert_after(n, Token::new(TokenType::Whitespace, " "), true);
                    n += 1;
                }
                n += 1;
            }

            let end_at: usize = identifiers.iter().map(|i| token_list.tokens[*i].value.len()+1).sum();
            let mut adjusted_offset: isize = 0;
            if self.wrap_after > 0 && end_at + self.offset > self.wrap_after && self.last_func_len > 0{
                adjusted_offset = 0-(self.last_func_len as isize)-1;
            }
            {
                self.indent += 1;
                let abs = adjusted_offset.abs() as usize;
                let offset = self.offset;
                self.offset = if abs > offset { 0 } else { offset - abs };

                if adjusted_offset < 0 {
                    token_list.insert_before(identifiers[0], self.nl(0));
                }
                let mut position = 0;
                let mut insert_count = 0;
                for mut tidx in identifiers {
                    tidx += insert_count;
                    let token = token_list.token_idx(Some(tidx)).unwrap();
                    position += token.value.len() + 1;
                    if self.wrap_after > 0 && position + self.offset > self.wrap_after {
                        if !token_list.insert_newline_before(tidx, self.nl(0)) {
                            insert_count += 1;
                        }
                        position = 0;
                    }
                }
                self.offset = offset;
                self.indent -= 1;
            }

        }
        parents.push(TokenType::IdentifierList);
        self.process_default(token_list, true, parents)

    }

    fn process_case(&mut self, token_list: &mut TokenList, mut parents: Vec<TokenType>) {
        let cases = token_list.get_case(false);
        // println!("cases: {:?}", cases);
        let cond = &cases[0];
        let first = cond.0[0];
        {
            let offset = self.get_offset("");
            self.offset += offset;
            {
                let extra = token_list.tokens.iter().take(first).map(|t| t.value.as_str()).collect::<Vec<&str>>().join("");
                let offset = self.get_offset(&extra);
                self.offset += offset;
                let mut insert_count = 0; // insert newline count
                for (cond, value) in cases.iter().skip(1) {
                    let token_idx = if cond.len() < 1 { value[0] } else { cond[0] };
                    if !token_list.insert_newline_before(token_idx+insert_count, self.nl(0)) {
                        insert_count += 1;
                    }
                }
                {
                    let n = "WHEN ".len();
                    self.offset += n;
                    parents.push(TokenType::Case);
                    self.process_default(token_list, true, parents);
                    self.offset -= n;
                }
                self.offset -= offset;
                let pattern = (TokenType::Keyword, vec!["END"]);
                let end_idx = token_list.token_next_by(&vec![], Some(&pattern), 0);
                if let Some(idx) = end_idx {
                    token_list.insert_newline_before(idx, self.nl(0));
                }
            }
            self.offset -= offset;
        }
    }

    fn process_values(&mut self, token_list: &mut TokenList) {
        token_list.insert_before(0, self.nl(0));
        let ttypes = vec![TokenType::Parenthesis];
        let mut tidx = token_list.token_next_by(&ttypes, None, 0);
        let first_idx = tidx;
        while let Some(idx) = tidx {
            let patterns = (TokenType::Punctuation, vec![","]);
            let pidx = token_list.token_next_by(&vec![], Some(&patterns), idx);
            if let Some(idx1) = pidx {
                let extra = token_list.take_value(first_idx.unwrap());
                if self.comma_first {
                    let offset = self.get_offset(&extra).saturating_sub(2);
                    token_list.insert_before(idx1, self.nl(offset as isize));
                } else {
                    // let extra = token_list.take_value(first_idx.unwrap());
                    let offset = self.get_offset(&extra);
                    let nl = self.nl(offset as isize);
                    token_list.insert_newline_after(idx1, nl, true);
                }
            }
            tidx = token_list.token_next_by(&ttypes, None, idx+1); 
        }
    }

    fn process_default(&mut self, token_list: &mut TokenList, split: bool, parents: Vec<TokenType>) {
        if split { self.split_statements(token_list); }
        self.split_kwds(token_list);
        // let mut remove_indexes = vec![];
        for token in token_list.tokens.iter_mut() {
            if token.is_group() {
                self.process_internal(&mut token.children, &token.typ, parents.clone());
                token.update_value();
                
                // if token.value.starts_with("\n") && i > 0 {
                //     remove_indexes.push(i-1);
                // }
            } else {
                self.prev_sql.push_str(&token.value);
            }
        }
        // remove_indexes.iter().enumerate().for_each(|(i, idx)| {token_list.tokens.remove(idx-i);});
    }
}


