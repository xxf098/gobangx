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

// TODO: add option
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

pub fn format(mut sql: &str, options: &mut formatter::FormatOption) -> String {
    let mut stack = engine::FilterStack::new();
    formatter::validate_options(options);
    formatter::build_filter_stack(&mut stack, options);
    if options.strip_whitespace { sql = sql.trim(); };
    let tokens = stack.format(sql, options.grouping);
    // for token in &tokens{
    //     println!("{:?}", token);
    // }
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
        formatter.identifier_case = "upper";
        let formatted_sql = format(sql, &mut formatter);
        assert_eq!(formatted_sql, "SELECT * FROM USERS LIMIT 10");
        let sql = "select * from \"t\".\"users\" limit 10";
        let formatted_sql = format(sql, &mut formatter);
        assert_eq!(formatted_sql, "SELECT * FROM \"t\".\"users\" LIMIT 10");
    }

    #[test]
    fn test_strip_ws() {
        let sql = "select     * from  users where  id  = 1;";
        let mut formatter = formatter::FormatOption::default();
        formatter.strip_whitespace = true;
        let formatted_sql = format(sql, &mut formatter);
        assert_eq!(formatted_sql, "select * from users where id = 1;");
    }


    #[test]
    fn test_strip_ws1() {
        let sql = "select\n* from      foo\n\twhere  ( 1 = 2 )\n";
        let mut formatter = formatter::FormatOption::default();
        formatter.strip_whitespace = true;
        let formatted_sql = format(sql, &mut formatter);
        assert_eq!(formatted_sql, "select * from foo where (1 = 2)");
    }
    
    #[test]
    fn test_preserve_ws() {
        let sql = "select\n* /* foo */  from bar ";
        let mut formatter = formatter::FormatOption::default();
        formatter.strip_whitespace = true;
        let formatted_sql = format(sql, &mut formatter);
        assert_eq!(formatted_sql, "select * /* foo */ from bar");
    }

    #[test]
    fn test_reindent_keywords() {
        let sql = "select * from foo union select * from bar;";
        let mut formatter = formatter::FormatOption::default_reindent();
        let formatted_sql = format(sql, &mut formatter);
        assert_eq!(formatted_sql, vec![
            "select *", 
            "from foo", 
            "union", 
            "select *", 
            "from bar;"].join("\n"))
    }

    #[test]
    fn test_reindent_keywords_between() {
        let sql = "and foo between 1 and 2 and bar = 3";
        let mut formatter = formatter::FormatOption::default_reindent();
        let formatted_sql = format(sql, &mut formatter);
        assert_eq!(formatted_sql, vec![
            "",
            "and foo between 1 and 2",
            "and bar = 3",
        ].join("\n"))
    }

    #[test]
    fn test_reindent_where() {
        let sql = "select * from foo where bar = 1 and baz = 2 or bzz = 3;";
        let mut formatter = formatter::FormatOption::default_reindent();
        let formatted_sql = format(sql, &mut formatter);
        // println!("{}", formatted_sql);
        assert_eq!(formatted_sql, vec![
            "select *",
            "from foo ",
            "where bar = 1",
            "  and baz = 2",
            "  or bzz = 3;",
        ].join("\n"))
    }
}
