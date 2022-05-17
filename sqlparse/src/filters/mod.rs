pub mod tokens;

use super::Token;

pub trait Filter {
    fn process(&self, token: &mut Token);
}
