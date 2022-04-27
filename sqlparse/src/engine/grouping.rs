use std::convert::From;
use crate::lexer::{Token, TokenList, tokenize};
use crate::tokens::TokenType;

const T_NUMERICAL: [TokenType; 3] = [TokenType::Number, TokenType::NumberInteger, TokenType::NumberFloat];
const T_STRING: [TokenType; 3] = [TokenType::String, TokenType::StringSingle, TokenType::StringSymbol];
const T_NAME: [TokenType; 2] = [TokenType::Name, TokenType::NamePlaceholder];

pub fn group(tokens: Vec<Token>) -> Vec<Token> {
    let mut token_list = TokenList::new(tokens);
    token_list.group();
    token_list.tokens
}

impl From<&str> for TokenList {
    
    fn from(sql: &str) -> Self {
        let tokens = tokenize(sql);
        TokenList::new(tokens)
    }
}

// TODO: GroupToken
impl TokenList {

    pub fn new(tokens: Vec<Token>) -> Self {
        // let group_tokens = tokens.into_iter().map(|t| t.into()).collect();
        Self { tokens: tokens }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    fn token_matching(&self, types: &[TokenType], pattern: Option<&(TokenType, Vec<&str>)>, start: usize, end: usize) -> Option<usize> {
        let pos = if types.len() > 0 {
            self.tokens[start..end].iter()
                .position(|token| types.iter().find(|t| **t == token.typ).is_some())
        } else if let Some(p) = pattern {
            self.tokens[start..end].iter()
                .position(|token| p.0 == token.typ && p.1.iter().find(|v| **v == token.normalized).is_some())
        } else {
            None
        };
        pos.map(|p| p+start)
    }

    fn token_matching_fn(&self, f: fn(&Token) -> bool, start: usize, end: usize, reverse: bool) -> Option<usize> {
        if reverse {
            self.tokens[start..end].iter().rposition(|token| f(token)).map(|p| p+start)
        } else {
            self.tokens[start..end].iter().position(|token| f(token)).map(|p| p+start)
        }
    }

    fn token_next_by(&self, types: &[TokenType], pattern: Option<&(TokenType, Vec<&str>)>,start: usize) -> Option<usize> {
        self.token_matching(types, pattern, start, self.tokens.len())
    }

    fn token_next(&self, idx: usize) -> Option<usize> {
        return self.token_matching_fn(|t| !t.is_whitespace(), idx, self.len(), false);
    }

    pub fn token_prev(&self, idx: usize) -> Option<usize> {
        if idx > self.len() || idx == 0 { None } 
        else { self.token_matching_fn(|t| !t.is_whitespace(), 0, idx, true) }
    }

    pub fn token_idx(&self, idx: Option<usize>) -> Option<&Token> {
        idx.map(|i| self.tokens.get(i)).flatten()
    }

    fn group_tokens(&mut self, group_type: TokenType, start: usize, end: usize) {
        let sub_tokens = self.tokens[start..end].to_vec();
        let group_token = vec![Token::new_parent(group_type, sub_tokens)];
        self.tokens.splice(start..end, group_token).for_each(drop);
    }

    fn group_identifier(&mut self) {
        // TODO: macro
        for token in self.tokens.iter_mut() {
            if token.children.len() > 0 {
                token.children.group_identifier();
            }
        }
        let ttypes = vec![TokenType::StringSymbol, TokenType::Name];
        let mut tidx = self.token_next_by(&ttypes, None, 0);
        while let Some(idx) = tidx {
            self.group_tokens(TokenType::Identifier, idx, idx +1);
            tidx = self.token_next_by(&ttypes, None, idx+1);
        }
    }   

    fn group_identifier_list(&mut self) {

        fn matcher(token: &Token) -> bool {
            token.typ == TokenType::Punctuation
        }

        fn valid(token: Option<&Token>) -> bool {
            let types = T_NUMERICAL.iter()
                .chain(&T_STRING)
                .chain(&T_NAME)
                .chain(&[TokenType::Keyword, TokenType::Comment, TokenType::Wildcard, 
                    TokenType::Function, TokenType::Case, TokenType::Identifier, 
                    TokenType::Comparison, TokenType::IdentifierList, TokenType::Operation])
                .map(|t| t.clone())
                .collect::<Vec<_>>();
        
            let patterns = (TokenType::Keyword, vec!["null", "role"]);
            return Token::imt(token, &types, Some(&patterns))
        }

        fn post(_tlist: &TokenList, pidx: usize, _tidx: usize, nidx: usize) -> (usize, usize) {
            (pidx, nidx)
        }

        group_internal(self, TokenType::IdentifierList, matcher, valid, valid, post, true, true)
    }

    fn group_where(&mut self) {
        let where_open = (TokenType::Keyword, vec!["WHERE"]);
        let where_close = (TokenType::Keyword, vec!["ORDER BY", "GROUP BY", "LIMIT", "UNION", "UNION ALL", "EXCEPT", "HAVING", "RETURNING", "INTO"]);
        let mut tidx = self.token_next_by(&vec![], Some(&where_open), 0);
        while let Some(idx) = tidx {
            let edix = self.token_next_by(&vec![], Some(&where_close), idx+1);
            let edix = edix.unwrap_or(self.tokens.len());
            // println!("idx {} eidx {}", idx, edix);
            self.group_tokens(TokenType::Where, idx, edix);
            tidx = self.token_next_by(&vec![], Some(&where_open), idx);
        }
    }

     fn group_comparison(&mut self) {

        fn matcher(token: &Token) -> bool {
            token.typ == TokenType::OperatorComparison
        }

        fn valid(token: Option<&Token>) -> bool {
            let types = vec![TokenType::Number, TokenType::NumberInteger, TokenType::NumberFloat, 
                TokenType::String, TokenType::StringSingle, TokenType::StringSymbol,
                TokenType::Name, TokenType::NamePlaceholder,
                TokenType::Function, TokenType::Identifier, TokenType::Operation, TokenType::TypedLiteral];
            let patterns = (TokenType::Parenthesis, vec!["(", ")"]);
            if Token::imt(token, &types, Some(&patterns)) {
                true
            } else if token.map(|t| t.typ == TokenType::Keyword && t.normalized == "NULL").unwrap_or(false) {
                true
            } else {
                false
            }
        }

        fn post(_tlist: &TokenList, pidx: usize, _tidx: usize, nidx: usize) -> (usize, usize) {
            (pidx, nidx)
        }

        group_internal(self, TokenType::Comparison, matcher, 
            valid, valid, post, false, true);
     }

     fn group_period(&mut self) {
        fn matcher(token: &Token) -> bool {
            token.typ == TokenType::Punctuation && token.value == "."
        }

        fn valid_prev(token: Option<&Token>) -> bool {
            let ttypes = vec![TokenType::Name, TokenType::StringSymbol, TokenType::SquareBrackets, TokenType::Identifier];
            Token::imt(token, &ttypes, None)
        }

        fn valid_next(token: Option<&Token>) -> bool {
            true
        }

        fn post(tlist: &TokenList, pidx: usize, tidx: usize, nidx: usize) -> (usize, usize) {
            let ttypes = vec![TokenType::Name, TokenType::StringSymbol, TokenType::Wildcard, TokenType::SquareBrackets, TokenType::Function];
            let next = tlist.token_idx(Some(nidx));
            let valid_next = Token::imt(next, &ttypes, None);
            if valid_next { (pidx, nidx) } else { (pidx, tidx) }
        }

        group_internal(self, TokenType::Identifier, matcher, 
            valid_prev, valid_next, post, true, true);
     }

     fn group(&mut self) {
        self.group_where();
        self.group_period();
        self.group_identifier();
        self.group_comparison();
        self.group_identifier_list();
     }

}

fn group_internal(
        tlist: &mut TokenList, 
        group_type: TokenType,
        matcher: fn(&Token) -> bool,
        valid_prev: fn(Option<&Token>) -> bool,
        valid_next: fn(Option<&Token>) -> bool,
        post: fn(tlist: &TokenList, pidx: usize, tidx: usize, nidx: usize) -> (usize, usize),
        extend: bool,
        recurse: bool,
    ) {
        let tidx_offset = 0;
        let mut pidx: Option<usize> = None;
        let mut prev_: Option<Token> = None;
        let mut idx = 0;
        while idx < tlist.len() {
            if idx < tidx_offset  {
                idx += 1;
                continue
            }
           
            if tlist.tokens[idx].is_whitespace() {
                idx += 1;
                continue
            }
            
            let token = &mut tlist.tokens[idx];
            if recurse && token.is_group() && token.typ != group_type {
                group_internal(&mut token.children, group_type.clone(), matcher, valid_prev, valid_next, post, extend, recurse);
                std::mem::drop(token)
            }

            let token = &tlist.tokens[idx];
            if matcher(token) {
                let nidx = tlist.token_next(idx+1);
                let next_ = tlist.token_idx(nidx);
                if pidx.is_some() && prev_.is_some() && valid_prev(prev_.as_ref()) && valid_next(next_) {
                    let (from_idx, to_idx) = post(&tlist, pidx.unwrap(), idx, nidx.unwrap());
                    tlist.group_tokens(group_type.clone(), from_idx, to_idx+1);
                    pidx = Some(from_idx);
                    prev_ = tlist.token_idx(pidx).map(|t| t.clone());
                    // idx += 1;
                    continue
                }
            }

            pidx = Some(idx);
            prev_ = Some(token.clone());
            idx += 1;
        }

}


#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_group_where1() {
        let sql = "select * from users where id > 10;";
        let tokens = tokenize(sql);
        let mut tokens = TokenList::new(tokens);
        tokens.group_where();
        println!("{:?}", tokens.tokens);
    }

    #[test]
    fn test_group_comparison() {
        let sql = "select * from users where id > 0;";
        let tokens = tokenize(sql);
        let mut token_list = TokenList::new(tokens);
        token_list.group_comparison();
        assert_eq!(token_list.tokens[10].typ, TokenType::Comparison)
        // for token in token_list.tokens {
        //     println!("{:?}", token);
        // }
       
    }

    #[test]
    fn test_group_comparison1() {
        let sql = "select * from users where id > 0;";
        let mut token_list = TokenList::from(sql);
        token_list.group_where();
        token_list.group_identifier();
        token_list.group_comparison();
        // assert_eq!(token_list.tokens[8].typ, TokenType::Where);
        for token in token_list.tokens {
            println!("{:?}", token);
        }
    }

    #[test]
    fn test_group_fn() {
        let sql = "select * from users where id > 0;";
        let mut token_list = TokenList::from(sql);
        token_list.group();
        assert_eq!(token_list.tokens[8].typ, TokenType::Where);
    }

    #[test]
    fn test_token_prev() {
        let sql= "select * from ";
        let token_list = TokenList::from(sql);
        let t = token_list.token_prev(token_list.len());
        let t = token_list.token_idx(t).unwrap();
        assert_eq!(t.value, "where");
    }

    #[test]
    fn test_group_period() {
        // TODO: select * from sch.user
        let sql = "select * from sch.account";
        let mut token_list = TokenList::from(sql);
        token_list.group_period();
       assert_eq!(token_list.tokens[6].typ, TokenType::Identifier);
       assert_eq!(token_list.tokens[6].value, "sch.account");

    }
}