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

    fn token_matching(&self, types: &[TokenType], patterns: &HashMap<TokenType, Vec<&str>>,start: usize, end: usize) -> Option<usize> {
        if types.len() > 0 {
            return self.tokens[start..end].iter()
            .position(|token| types.iter().find(|t| **t == token.typ).is_some())
        }
        if patterns.len() > 0 {
            return self.tokens[start..end].iter()
                .position(|token| {
                    patterns.iter().find(|(k, vals)| **k == token.typ && vals.iter().find(|v| **v == token.value).is_some()).is_some()
                })
        }
        None
    }

    // tuple
    fn token_next_by(&self, types: &[TokenType], patterns: &HashMap<TokenType, Vec<&str>>,start: usize) -> Option<usize> {
        self.token_matching(types, patterns, start, self.tokens.len())
    }

    fn group_tokens(&mut self, group_type: TokenType, start: usize, end: usize) {
        let sub_tokens = self.tokens[start..end].to_vec();
        let group_token = vec![Token::new_parent(group_type, sub_tokens)];
        self.tokens.splice(start..end, group_token).for_each(drop);
    }

    fn group_identifier(&mut self) {
        let ttypes = vec![TokenType::StringSymbol, TokenType::Name];
        let patterns = HashMap::new();
        let mut tidx = self.token_next_by(&ttypes, &patterns, 0);
        while let Some(idx) = tidx {
            self.group_tokens(TokenType::Identifier, idx, idx +1);
            tidx = self.token_next_by(&ttypes, &patterns, idx+1);
        }
    }

    fn group_where(&mut self) {
        let mut where_open = HashMap::new();
        where_open.insert(TokenType::Keyword, vec!["WHERE"]);
        let mut tidx = self.token_next_by(&vec![], &where_open, 0);
        while let Some(idx) = tidx {
            tidx = self.token_next_by(&vec![], &where_open, 0);
        }
    } 

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
}