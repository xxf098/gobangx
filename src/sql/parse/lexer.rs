use super::keywords::{sql_regex, is_keyword};
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
        let mut forawrd = 0;
        for rt in &regs {
            let t = &sql[index..];
            let opt = match rt.capture {
                Some(i) =>  rt.reg.captures(t).map(|c| c.get(i)).flatten(),
                None =>  rt.reg.find(t)
            };
            if opt.is_none() || opt.unwrap().start() != 0 {
                continue
            }
            let v = opt.unwrap().as_str();
            forawrd = v.len();
            let typ = match rt.typ {
                TokenType::KeywordRaw => is_keyword(v),
                _ => rt.typ.clone()
            };
            let t = Token::new(typ, v.to_string());
            tokens.push(t);
            break;
        }
        if forawrd == 0 {
            break;
        }
        index += forawrd
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
        let sql = "select * from users;";
        let tokens = get_tokens(sql);
        println!("{}", tokens.len());
        println!("{:?}", tokens);
    }

    #[test]
    fn test_get_tokens2() {
        let sql = "SELECT article, MAX(price) AS price FROM   shop GROUP BY article ORDER BY article;";
        let tokens = get_tokens(sql);
        println!("{}", tokens.len());
        println!("{:?}", tokens);
    }
   
}
