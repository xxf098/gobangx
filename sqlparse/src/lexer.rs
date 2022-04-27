use super::keywords::{sql_regex, is_keyword};
use super::tokens::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub typ: TokenType,
    pub value: String,
    pub children: TokenList,
    pub normalized: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenList {
    pub tokens: Vec<Token>,
}

// impl Iterator for TokenList {
// }

impl Token {

    pub fn new(typ: TokenType, value: String) -> Self {
        let token_list = TokenList::new(vec![]);
        let normalized = if typ == TokenType::Keyword { value.to_uppercase() } else { value.clone() };
        Self { typ, value, children: token_list, normalized }
    }

    pub fn new_parent(typ: TokenType, children: Vec<Token>) -> Self {
        let value = children.iter().map(|child| child.value.as_ref()).collect::<Vec<_>>().join("");
        let token_list = TokenList::new(children);
        let normalized = if typ == TokenType::Keyword { value.to_uppercase() } else { value.clone() };
        Self { typ, value, children: token_list, normalized }
    }

    pub fn is_whitespace(&self) -> bool {
        self.typ == TokenType::Whitespace
    }

    pub fn is_keyword(&self) -> bool {
        self.typ == TokenType::Keyword        
    }

    pub fn is_group(&self) -> bool {
        self.children.len() > 0
    }

    // comparisons token
    pub fn imt(token: Option<&Token>, types: &[TokenType], pattern: Option<&(TokenType, Vec<&str>)>) -> bool {
        if token.is_none() {
            return false
        }
        let token = token.unwrap();
        if types.len() > 0 {
            return types.iter().find(|typ| **typ == token.typ).is_some()
        } else if let Some(p) = pattern {
            return p.0 == token.typ && p.1.iter().find(|v| **v == token.normalized).is_some()
        } else {
            return false
        }
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
            let i = index.saturating_sub(rt.backward);
            let t = &sql[i..];
         
            let opt = match rt.capture {
                Some(i) =>  rt.reg.captures(t).map(|c| c.get(i)).flatten(),
                None =>  rt.reg.find(t)
            };
            if opt.is_none() || opt.unwrap().start() != rt.backward {
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


#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_group_token() {
        let v = vec![0,1,2,3];
        let sub = v[1..2].to_vec();
        println!("{:?}", sub);
    }
}
