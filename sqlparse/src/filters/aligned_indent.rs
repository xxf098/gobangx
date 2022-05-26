use super::TokenListFilter;
use crate::lexer::{Token, TokenList};
use crate::tokens::TokenType;

pub struct AlignedIndentFilter {
    pub n: String,
    pub offset: usize,
    pub indent: usize,
    pub chr: String,
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
        }
    }

    fn nl(&self, offset: usize) -> Token {
        let indent = self.indent * (2 + self.max_kwd_len);
        let i = self.max_kwd_len + offset + indent + self.offset;
        let white = format!("{}{}", self.n, self.chr.repeat(i));
        Token::new(TokenType::Whitespace, &white)
    }

    fn process_internal(&mut self, token_list: &mut TokenList) {
        
    }

    fn process_default(&mut self, token_list: &mut TokenList) {
        
    }
}