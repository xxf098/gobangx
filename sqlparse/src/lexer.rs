use super::keywords::{sql_regex, is_keyword, RegexToken};
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

    pub fn get_name(&self) -> Option<&str> {
        self.get_alias().or(self.get_real_name())
    }

    // Returns the real name (object name) of this identifier.
    pub fn get_real_name(&self) -> Option<&str> {
        if !self.is_group() {
            return None
        }
        let patterns = (TokenType::Punctuation, vec!["."]);
        let children = &self.children;
        let dot_idx = children.token_next_by(&vec![], Some(&patterns), 0);
        children.get_first_name(dot_idx, false, false, true)
    }

    pub fn get_parent_name(&self) -> Option<&str> {
        let pattern = (TokenType::Punctuation, vec!["."]);
        let children = &self.children;
        let dot_idx = children.token_next_by(&vec![], Some(&pattern), 0);
        let prev_idx = dot_idx.map(|idx| children.token_prev(idx)).flatten();
        let prev = children.token_idx(prev_idx);
        prev.map(|p| remove_quotes(&p.value))
    }

    pub fn get_alias(&self) -> Option<&str> {
        if self.typ != TokenType::Identifier {
            return None
        }
        //  "name AS alias"
        let patterns = (TokenType::Keyword, vec!["AS"]);
        let kw_idx = self.children.token_next_by(&vec![], Some(&patterns), 0);
        if let Some(idx) = kw_idx {
            return self.children.get_first_name(Some(idx+1), false, true, false)
        }
        // "name alias" or "complicated column expression alias"
        let ttypes = vec![TokenType::Whitespace];
        let ws_idx = self.children.token_next_by(&ttypes, None, 0);
        if self.children.len() > 2 && ws_idx.is_some() {
            return self.children.get_first_name(Some(0), true, false, false)
        }
        None
    }

}

pub(crate) fn remove_quotes(mut s: &str) -> &str {
    if s.starts_with("\"") {
        s = s.trim_start_matches("\"");
        s.trim_end_matches("\"")
    } else if s.starts_with("'") {
        s = s.trim_start_matches("'");
        s.trim_end_matches("'")
    } else {
        s
    }
}

pub fn tokenize(sql: &str) -> Vec<Token> {
    let regs = sql_regex(false);
    tokenize_internal(sql, &regs)
}

pub fn tokenize_internal(sql: &str, regs: &[RegexToken]) -> Vec<Token> {
    let mut tokens = vec![];
    let mut index = 0;
    let sql_len = sql.len();
    while index < sql_len {
        let mut forawrd = 0;
        for rt in regs {
            let i = index.saturating_sub(rt.backward);
            let t = &sql[i..];
         
            let opt = match rt.capture {
                Some(i) => rt.reg.captures(t).map(|c| c.get(i)).flatten(),
                None => rt.reg.find(t)
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
    use std::time::Instant;

    #[test]
    fn test_get_tokens2() {
        let sql = "SELECT article, MAX(price) AS price FROM   shop GROUP BY article ORDER BY article;";
        let now = Instant::now();
        let tokens = tokenize(sql);
        let elapsed = now.elapsed().as_millis();
        println!("elapsed: {}ms", elapsed);
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
