use super::keywords::{sql_regex, is_keyword};
use super::tokens::TokenType;

#[derive(Debug, Clone)]
pub struct Token {
    typ: TokenType,
    value: String,
}

#[derive(Debug, Clone)]
pub struct GroupToken {
    typ: TokenType,
    tokens: Vec<Token>,
}

impl From<Token> for GroupToken {
    fn from(token: Token) -> Self {
        Self { typ: token.typ.clone(), tokens: vec![token] }
    }
}

impl Token {

    pub fn new(typ: TokenType, value: String) -> Self {
        Self { typ, value }
    }
}

pub fn tokenize(sql: &str) -> Vec<Token> {
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


pub struct TokenList {
    tokens: Vec<Token>,
}

impl TokenList {

    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    fn token_matching(&self, types: &[TokenType], start: usize, end: usize) -> Option<usize> {
        self.tokens[start..end].iter().enumerate()
            .position(|(_, token)| types.iter().find(|t| **t == token.typ).is_some())
    }

    pub fn token_next_by(&self, types: &[TokenType]) -> Option<usize> {
        self.token_matching(types, 0, self.tokens.len())
    }

    pub fn group_tokens(&mut self) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tokens1() {
        let sql = "select * from users;";
        let tokens = tokenize(sql);
        let tokens = TokenList::new(tokens);
        let next_token = tokens.token_next_by(&[TokenType::KeywordDML]);
        assert_eq!(next_token, Some(0))
    }

    #[test]
    fn test_get_tokens2() {
        let sql = "SELECT article, MAX(price) AS price FROM   shop GROUP BY article ORDER BY article;";
        let tokens = tokenize(sql);
        println!("{}", tokens.len());
        println!("{:?}", tokens);
    }
   
    #[test]
    fn test_get_tokens3() {
        let sql = "SELECT Orders.OrderID, Customers.CustomerName FROM Orders INNER JOIN Customers ON Orders.CustomerID = Customers.CustomerID;";
        let tokens = tokenize(sql);
        println!("{}", tokens.len());
        println!("{:?}", tokens);
        assert_eq!(tokens.len(), 31);
    }

    #[test]
    fn test_get_tokens4() {
        let sql = "SELECT OrderID, Quantity, CASE WHEN Quantity > 30 THEN 'The quantity is greater than 30' WHEN Quantity = 30 THEN 'The quantity is 30' ELSE 'The quantity is under 30' END AS QuantityText FROM OrderDetails;";
        let tokens = tokenize(sql);
        println!("{}", tokens.len());
        println!("{:?}", tokens);
        assert_eq!(tokens.len(), 48);
    }
}
