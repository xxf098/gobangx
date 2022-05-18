pub mod tokens;
pub mod others;

use super::Token;

pub trait Filter: Send+Sync {
    fn process(&self, token: &mut Token, depth: usize);
}
