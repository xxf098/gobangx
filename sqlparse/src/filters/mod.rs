pub mod tokens;

use super::Token;

pub trait Filter: Send+Sync {
    fn process(&self, token: &mut Token);
}
