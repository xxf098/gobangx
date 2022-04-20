use std::collections::HashMap;
use crate::sql::parse::lexer::{Token};
use crate::sql::parse::tokens::TokenType;

pub fn group(stmt: Vec<Token>) -> Vec<Token> {
    // stmt.into_iter().map(|s| s.into()).collect::<Vec<_>>()
    vec![]
}

#[derive(Debug)]
pub struct TokenList {
    pub tokens: Vec<Token>,
}

// TODO: GroupToken
impl TokenList {

    pub fn new(tokens: Vec<Token>) -> Self {
        // let group_tokens = tokens.into_iter().map(|t| t.into()).collect();
        Self { tokens: tokens }
    }

    fn token_matching(&self, types: &[TokenType], pattern: Option<&(TokenType, Vec<&str>)>, start: usize, end: usize) -> Option<usize> {
        let pos = if types.len() > 0 {
            self.tokens[start..end].iter()
                .position(|token| types.iter().find(|t| **t == token.typ).is_some())
        } else if let Some(p) = pattern {
            self.tokens[start..end].iter()
                .position(|token| p.0 == token.typ && p.1.iter().find(|v| **v == token.value.to_uppercase()).is_some())
        } else {
            None
        };
        pos.map(|p| p+start)
    }

    // tuple
    fn token_next_by(&self, types: &[TokenType], pattern: Option<&(TokenType, Vec<&str>)>,start: usize) -> Option<usize> {
        self.token_matching(types, pattern, start, self.tokens.len())
    }

    fn group_tokens(&mut self, group_type: TokenType, start: usize, end: usize) {
        let sub_tokens = self.tokens[start..end].to_vec();
        let group_token = vec![Token::new_parent(group_type, sub_tokens)];
        self.tokens.splice(start..end, group_token).for_each(drop);
    }

    fn group_identifier(&mut self) {
        let ttypes = vec![TokenType::StringSymbol, TokenType::Name];
        let mut tidx = self.token_next_by(&ttypes, None, 0);
        while let Some(idx) = tidx {
            self.group_tokens(TokenType::Identifier, idx, idx +1);
            tidx = self.token_next_by(&ttypes, None, idx+1);
        }
    }   

    fn group_where(&mut self) {
        let where_open = (TokenType::Keyword, vec!["WHERE"]);
        let where_close = (TokenType::Keyword, vec!["ORDER BY", "GROUP BY", "LIMIT", "UNION", "UNION ALL", "EXCEPT", "HAVING", "RETURNING", "INTO"]);
        let mut tidx = self.token_next_by(&vec![], Some(&where_open), 0);
        while let Some(idx) = tidx {
            let edix = self.token_next_by(&vec![], Some(&where_close), idx+1);
            let edix = edix.unwrap_or(self.tokens.len()-1);
            println!("idx {} eidx {}", idx, edix);
            self.group_tokens(TokenType::Where, idx, edix);
            tidx = self.token_next_by(&vec![], Some(&where_open), idx);
        }
    }

     // group_comparison

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parse::lexer::{tokenize};

    #[test]
    fn test_group_identifier() {
        let sql = "select * from users;";
        let tokens = tokenize(sql);
        let mut tokens = TokenList::new(tokens);
        tokens.group_identifier();
        println!("{:?}", tokens.tokens);
    }

    #[test]
    fn test_group_where() {
        let sql = "select * from users where id > 10 limit 10;";
        let tokens = tokenize(sql);
        let mut tokens = TokenList::new(tokens);
        tokens.group_where();
        println!("{:?}", tokens.tokens);
    }
}