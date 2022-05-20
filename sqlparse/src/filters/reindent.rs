use super::TokenListFilter;
use crate::lexer::{Token, TokenList};
use crate::tokens::TokenType;

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

    fn process(&self, token_list: &mut TokenList) {

    }
}


impl ReindentFilter {

    fn new(width: usize, chr: &str, wrap_after: usize, n: &str, 
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

    fn split_kwds(&self, token_list: &mut TokenList) {
        
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

    fn process_default(&self, token_list: &mut TokenList) {
        self.split_statements(token_list);

    }
    
}


