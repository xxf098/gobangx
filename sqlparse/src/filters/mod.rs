pub mod tokens;
pub mod others;
pub mod reindent;
pub mod aligned_indent;

use super::{Token, TokenList, TokenType};
pub use tokens::{IdentifierCaseFilter, KeywordCaseFilter};
pub use others::StripWhitespaceFilter;
pub use reindent::ReindentFilter;
pub use aligned_indent::AlignedIndentFilter;

pub trait Filter: Send+Sync {
    fn process(&self, token: &mut Token);
}

// FIXME
pub trait StmtFilter: Send+Sync {
    fn process(&self, tokens: &mut Vec<Token>);
}


pub trait TokenListFilter: Send+Sync {
    fn process(&mut self, token_list: &mut TokenList);
}

const SPLIT_WORDS: [&str; 12] = ["FROM", "AND", "OR", "GROUP BY", 
    "ORDER BY", "UNION", "VALUES", "SET", "BETWEEN", "EXCEPT", "HAVING", "LIMIT"];

const SPLIT_WORDS_ALIGN: [&str; 14] = ["FROM", "ON", "WHERE", "AND", "OR",
"GROUP BY", "ORDER BY","UNION", "VALUES", 
"SET", "BETWEEN", "EXCEPT", "HAVING", "LIMIT"];

fn next_token(token_list: &TokenList, idx: usize) -> Option<usize> {
    next_token_internal(token_list, idx, &SPLIT_WORDS)
}

fn next_token_align(token_list: &TokenList, idx: usize) -> Option<usize> {
    next_token_internal(token_list, idx, &SPLIT_WORDS_ALIGN)
}

fn next_token_internal(token_list: &TokenList, idx: usize, split_words: &[&str]) -> Option<usize> {
    let mut tidx = token_list.token_next_by_fn(|t| match t.typ {
        TokenType::Keyword => split_words.iter().find(|w| **w == t.normalized).is_some() || t.normalized.ends_with("STRAIGHT_JOIN") || t.normalized.ends_with("JOIN"),
        _ => false
    } , idx);
    let token = token_list.token_idx(tidx);
    if token.map(|t| t.normalized == "BETWEEN").unwrap_or(false) {
        tidx = next_token_internal(token_list, tidx.unwrap()+1, split_words);
        let token = token_list.token_idx(tidx);
        if token.map(|t| t.normalized == "AND").unwrap_or(false) {
            tidx = next_token_internal(token_list, tidx.unwrap()+1, split_words);
        } 
    }
    tidx
}