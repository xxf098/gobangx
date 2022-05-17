mod engine;
mod lexer;
mod keywords;
mod tokens;
mod formatter;
mod filters;


pub use tokens::TokenType;
pub use lexer::{Token, TokenList};
pub use formatter::{FormatOption, validate_options};

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

pub fn format(sql: &str, options: formatter::FormatOption) -> String {
    let stack = engine::FilterStack::new();
    let options = formatter::validate_options(options);
    let stack = formatter::build_filter_stack(stack, &options);
    let tokens = stack.format(sql);
    tokens.iter().map(|token| token.value.as_str()).collect()
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
        let sql= "SELECT article, MAX(price) AS price FROM shop GROUP BY article ORDER BY article;";
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
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].typ, TokenType::Identifier);
        println!("{:?}", tokens);
    }

    #[test]
    fn test_parser3() {
        let sql= "SELECT COUNT(CustomerID), Country FROM Customers GROUP BY Country HAVING COUNT(CustomerID) > 5 ORDER BY COUNT(CustomerID) DESC;";
        let p = Parser::default();
        let now = Instant::now();
        let _tokens = p.parse(sql);
        let elapsed = now.elapsed();
        println!("elapsed: {}ms", elapsed.as_millis());
    }

    #[test]
    fn test_format() {
        let sql = "select * from users limit 10";
        let mut formatter = formatter::FormatOption::default();
        formatter.keyword_case = "upper";
        let formatted_sql = format(sql, formatter);
        println!("{}", formatted_sql);
    }

}
