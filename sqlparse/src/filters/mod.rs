pub mod tokens;
pub mod others;
pub mod reindent;

use super::Token;
pub use tokens::{IdentifierCaseFilter, KeywordCaseFilter};
pub use others::StripWhitespaceFilter;

pub trait Filter: Send+Sync {
    fn process(&self, token: &mut Token);
}


pub trait StmtFilter: Send+Sync {
    fn process(&self, tokens: &mut Vec<Token>);
}
