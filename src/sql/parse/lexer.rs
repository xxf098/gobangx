use super::keywords::sql_regex;
use super::tokens::TokenType;

#[derive(Debug, Clone)]
pub struct Token {
    typ: TokenType,
    value: String,
}

impl Token {

    pub fn new(typ: TokenType, value: String) -> Self {
        Self { typ, value }
    }
}

fn get_tokens(sql: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut index = 0;
    let sql_len = sql.len();
    let regs = sql_regex();
    while index < sql_len {
        for rt in &regs {
            let t = &sql[index..];
            let opt = rt.reg.find(t);
            if opt.is_none() || opt.unwrap().start() != 0 {
                continue
            }
            let v = opt.unwrap().as_str();
            index += v.len();
            let t = Token::new(rt.typ.clone(), v.to_string());
            tokens.push(t);
        }
    };
    tokens
}

pub fn tokenize(sql: &str) {

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tokens1() {
        let sql = "SELECT * FROM users;";
        let tokens = get_tokens(sql);
        println!("{}", tokens.len());
        println!("{:?}", tokens);
    }
   
}
