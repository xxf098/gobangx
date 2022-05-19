pub mod tokens;
pub mod others;
pub mod reindent;

use super::{Token, TokenList};
pub use tokens::{IdentifierCaseFilter, KeywordCaseFilter};
pub use others::StripWhitespaceFilter;

pub trait Filter: Send+Sync {
    fn process(&self, token: &mut Token);
}

// FIXME
pub trait StmtFilter: Send+Sync {
    fn process(&self, tokens: &mut Vec<Token>);
}


pub trait TokenListFilter: Send+Sync {
    fn process(&self, token_list: &mut TokenList);
}