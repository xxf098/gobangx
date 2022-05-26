pub mod tokens;
pub mod others;
pub mod reindent;
pub mod aligned_indent;

use super::{Token, TokenList};
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