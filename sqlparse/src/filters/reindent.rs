use super::TokenListFilter;
use crate::lexer::TokenList;
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

    fn process_default(token_list: &mut TokenList) {

    }

    fn split_statements(token_list: &mut TokenList) {
        let ttypes = vec![TokenType::KeywordDML, TokenType::KeywordDDL];
        let tidx = token_list.token_next_by(&ttypes, None, 0);
        while let Some(idx) = tidx {
            let pidx = token_list.token_prev(idx, false);
        }
    }
    
}


