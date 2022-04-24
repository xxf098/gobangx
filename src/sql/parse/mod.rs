pub mod engine;
pub mod lexer;
pub mod keywords;
pub mod tokens;

pub use tokens::TokenType;
pub use lexer::Token;

pub fn parse(sql: &str) -> Vec<Token> {
    let stack = engine::FilterStack::new(true);
    stack.run(sql)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let sql = "select * from users where id > 1;";
        let tokens = parse(sql);
        for token in tokens {
            println!("{:?}", token);
        }
    }
}
