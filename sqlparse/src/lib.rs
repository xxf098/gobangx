mod engine;
mod lexer;
mod keywords;
mod tokens;


pub use tokens::TokenType;
pub use lexer::{Token, TokenList};

pub fn parse(sql: &str) -> Vec<Token> {
    let stack = engine::FilterStack::new(true);
    stack.run(sql)
}

pub fn parse_no_grouping(sql: &str) -> Vec<Token> {
    let stack = engine::FilterStack::new(false);
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

    #[test]
    fn test_parse_identifier() {
        let sql = "select * from sch.users;";
        let tokens = parse(sql);
        // let tokens = parse_no_grouping(sql);
        for token in tokens {
            println!("{:?} {}", token.typ, token.value);
        }
    }
}
