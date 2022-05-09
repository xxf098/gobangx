mod engine;
mod lexer;
mod keywords;
mod tokens;


pub use tokens::TokenType;
pub use lexer::{Token, TokenList};

pub struct Parser {
    stack: engine::FilterStack,
}

impl Default for Parser {
    fn default() -> Self {
        Self { stack: engine::FilterStack::new() }
    }
}


impl Parser {

    pub fn new() -> Self {
        Self { stack: engine::FilterStack::new() }
    }

    pub fn parse(&self, sql: &str) -> Vec<Token> {
        self.stack.run(sql, true)
    }

    pub fn parse_no_grouping(&self, sql: &str) -> Vec<Token> {
        self.stack.run(sql, false)
    }
}

// only for test
pub fn parse(sql: &str) -> Vec<Token> {
    let stack = engine::FilterStack::new();
    stack.run(sql, true)
}

// only for test
pub fn parse_no_grouping(sql: &str) -> Vec<Token> {
    let stack = engine::FilterStack::new();
    stack.run(sql, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

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

    #[test]
    fn test_parser1() {
        let sql= "select l from test";
        let p = Parser::default();
        let now = Instant::now();
        let _tokens = p.parse(sql);
        let elapsed = now.elapsed();
        println!("elapsed: {}ms", elapsed.as_millis());
    }


    #[test]
    fn test_parser2() {
        let sql= "s";
        let p = Parser::default();
        let tokens = p.parse(sql);
        println!("{:?}", tokens);
    }

}
