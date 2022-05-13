use std::convert::From;
use std::fmt;
use crate::lexer::{Token, TokenList, tokenize, remove_quotes};
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

impl std::fmt::Display for TokenList {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for token in &self.tokens {
            writeln!(f, "{:?}", token)?;
        };
        Ok(())
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

    pub fn token_next_by(&self, types: &[TokenType], pattern: Option<&(TokenType, Vec<&str>)>,start: usize) -> Option<usize> {
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

    pub fn extend(&mut self, tokens: Vec<Token>) {
        self.tokens.extend(tokens)
    }

    // extend: flatten tokens
    fn group_tokens(&mut self, group_type: TokenType, start: usize, end: usize, extend: bool) {
        if extend && self.tokens[start].typ == group_type {
            let start_idx = start;
            let sub_tokens = self.tokens[start_idx+1..end].to_vec();
            let start = &mut self.tokens[start_idx];
            start.children.extend(sub_tokens);
            self.tokens.splice(start_idx+1..end, []).for_each(drop);
            return
        }
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
            self.group_tokens(TokenType::Identifier, idx, idx +1, false);
            tidx = self.token_next_by(&ttypes, None, idx+1);
        }
    }   

    fn group_identifier_list(&mut self) {

        fn matcher(token: &Token) -> bool {
            token.typ == TokenType::Punctuation && token.value == ","
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

        fn post(_tlist: &mut TokenList, pidx: usize, _tidx: usize, nidx: usize) -> (usize, usize) {
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
            self.group_tokens(TokenType::Where, idx, edix, false);
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

        fn post(_tlist: &mut TokenList, pidx: usize, _tidx: usize, nidx: usize) -> (usize, usize) {
            (pidx, nidx)
        }

        group_internal(self, TokenType::Comparison, matcher, valid, valid, post, false, true);
     }

     fn group_operator(&mut self) {

        fn matcher(token: &Token) -> bool {
            token.typ == TokenType::Operator || token.typ == TokenType::Wildcard
        }

        fn valid(token: Option<&Token>) -> bool {
            let mut types = T_NUMERICAL.iter()
                .chain(&T_STRING)
                .chain(&T_NAME)
                .map(|t| t.clone())
                .collect::<Vec<_>>();
            types.extend(vec![TokenType::SquareBrackets, TokenType::Parenthesis, TokenType::Function, 
                    TokenType::Identifier, TokenType::Operation, TokenType::TypedLiteral]);
            Token::imt(token, &types, None) || 
                token.map(|t| t.typ == TokenType::Keyword && (t.value == "CURRENT_DATE" || t.value == "CURRENT_TIME" || t.value == "CURRENT_TIMESTAMP")).unwrap_or(false)
        }

        fn post(tlist: &mut TokenList, pidx: usize, tidx: usize, nidx: usize) -> (usize, usize) {
            tlist.tokens[tidx].typ = TokenType::Operator; 
            (pidx, nidx)
        }

        group_internal(self, TokenType::Operation, matcher, valid, valid, post, false, true)
     }

     // schema.table
     fn group_period(&mut self) {
        fn matcher(token: &Token) -> bool {
            token.typ == TokenType::Punctuation && token.value == "."
        }

        fn valid_prev(token: Option<&Token>) -> bool {
            let ttypes = vec![TokenType::Name, TokenType::StringSymbol, TokenType::SquareBrackets, TokenType::Identifier];
            Token::imt(token, &ttypes, None)
        }

        fn valid_next(_token: Option<&Token>) -> bool {
            true
        }

        fn post(tlist: &mut TokenList, pidx: usize, tidx: usize, nidx: usize) -> (usize, usize) {
            let ttypes = vec![TokenType::Name, TokenType::StringSymbol, TokenType::Wildcard, TokenType::SquareBrackets, TokenType::Function];
            let next = tlist.token_idx(Some(nidx));
            let valid_next = Token::imt(next, &ttypes, None);
            if valid_next { (pidx, nidx) } else { (pidx, tidx) }
        }

        group_internal(self, TokenType::Identifier, matcher, 
            valid_prev, valid_next, post, true, true);
     }

    fn group_as(&mut self) {

        fn matcher(token: &Token) -> bool {
            token.is_keyword() && token.normalized == "AS"
        }

        fn valid_prev(token: Option<&Token>) -> bool {
            token.map(|t| t.normalized == "NULL" || !t.is_keyword()).unwrap_or(false)
        }

        fn valid_next(token: Option<&Token>) -> bool {
            let ttypes = vec![TokenType::DML, TokenType::DDL, TokenType::CTE];
            !Token::imt(token, &ttypes, None)
        }

        fn post(_tlist: &mut TokenList, pidx: usize, _tidx: usize, nidx: usize) -> (usize, usize) {
            (pidx, nidx)
        }

        group_internal(self, TokenType::Identifier, matcher, valid_prev, valid_next, post, true, true);
    }

    //  Group together Identifier and Asc/Desc token
    fn group_order(&mut self) {
        let ttypes = vec![TokenType::KeywordOrder];
        let mut tidx = self.token_next_by(&ttypes, None, 0);
        while let Some(idx) = tidx {
            let pidx = self.token_prev(idx);
            let prev = self.token_idx(pidx);
            let ttypes = vec![TokenType::Identifier, TokenType::Number];
            if Token::imt(prev, &ttypes, None) {
                self.group_tokens(TokenType::Identifier, pidx.unwrap(), idx+1, false);
                tidx = pidx;
            }
            tidx = self.token_next_by(&ttypes, None, tidx.unwrap()+1);
        }

    }

    fn group(&mut self) {
        self.group_where();
        self.group_period();
        self.group_identifier();
        self.group_order();
        self.group_operator();
        self.group_comparison();
        self.group_as();
        self.group_identifier_list();
    }

    pub fn get_first_name(&self, idx: Option<usize>, reverse: bool, keywords: bool, real_name: bool) -> Option<&str> {
        let idx = idx.unwrap_or(0);
        let tokens = &self.tokens[idx..];
        let mut ttypes = vec![TokenType::Name, TokenType::Wildcard, TokenType::StringSymbol];
        if keywords {
            ttypes.push(TokenType::Keyword)
        }
        if reverse {
            for token in tokens.iter().rev() {
                if ttypes.iter().find(|typ| **typ == token.typ).is_some() {
                    return Some(remove_quotes(&token.value))
                } else if token.typ == TokenType::Identifier || token.typ == TokenType::Function {
                    return if real_name { token.get_real_name() } else { token.get_name() }
                }         
            }
        }
        for token in tokens {
            if ttypes.iter().find(|typ| **typ == token.typ).is_some() {
                return Some(remove_quotes(&token.value))
            } else if token.typ == TokenType::Identifier || token.typ == TokenType::Function {
                return if real_name { token.get_real_name() } else { token.get_name() }
            }         
        }
        None
    }

}

fn group_matching(tlist: &mut TokenList, typ: &TokenType, open: &str, close: &str) {
    // Groups Tokens that have beginning and end.
    let mut opens = vec![];
    let mut tidx_offset = 0;
    for (idx, token) in tlist.tokens.iter_mut().enumerate() {
        let tidx = idx - tidx_offset;
        if token.is_whitespace() {
            continue
        }
        if token.is_group() && token.typ != *typ {
            group_matching(&mut token.children, typ, open, close);
            continue
        }
        if token.value == open {
            opens.push(tidx);
        } else if token.value == close {
            if opens.len() < 1 {
                continue
            }
            let open_idx = opens[opens.len()-1];
            opens.truncate(opens.len()-1);
            let close_idx = tidx;
            std::mem::drop(token);
            tlist.group_tokens(typ.clone(), open_idx, close_idx, false);
            tidx_offset += close_idx - open_idx;
        }
    }
}

// TODO: interface Grouping
fn group_internal(
        tlist: &mut TokenList, 
        group_type: TokenType,
        matcher: fn(&Token) -> bool,
        valid_prev: fn(Option<&Token>) -> bool,
        valid_next: fn(Option<&Token>) -> bool,
        post: fn(tlist: &mut TokenList, pidx: usize, tidx: usize, nidx: usize) -> (usize, usize),
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
                    let (from_idx, to_idx) = post(tlist, pidx.unwrap(), idx, nidx.unwrap());
                    tlist.group_tokens(group_type.clone(), from_idx, to_idx+1, extend);
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
        let sql = "select * from sch.account";
        let mut token_list = TokenList::from(sql);
        token_list.group_period();
        assert_eq!(token_list.tokens[6].typ, TokenType::Identifier);
        assert_eq!(token_list.tokens[6].value, "sch.account");

        let sql = "select * from sch.user";
        let mut token_list = TokenList::from(sql);
        token_list.group_period();
        assert_eq!(token_list.tokens[6].typ, TokenType::Identifier);
        assert_eq!(token_list.tokens[6].value, "sch.user");

        let sql = "select * from sch.user as u";
        let mut token_list = TokenList::from(sql);
        token_list.group_period();
        token_list.group_as();
        assert_eq!(token_list.tokens[6].typ, TokenType::Identifier);
        assert_eq!(token_list.tokens[6].value, "sch.user as u");
    }

    #[test]
    fn test_group_order() {
        let sql = "select * from users order by id desc";
        let mut token_list = TokenList::from(sql);
        token_list.group();
        assert_eq!(token_list.tokens[10].typ, TokenType::Identifier);
        assert_eq!(token_list.tokens[10].value, "id desc");
    }

    #[test]
    fn test_get_real_name() {
        let sql = "select * from test.person as p where ";
        let mut token_list = TokenList::from(sql);
        token_list.group();
        let id = token_list.token_idx(Some(6)).unwrap();
        let real_name = id.get_real_name();
        let parent_name = id.get_parent_name();
        let alias = id.get_alias();
        assert_eq!(real_name, Some("person"));
        assert_eq!(parent_name, Some("test"));
        assert_eq!(alias, Some("p"));

        let sql = "select * from test.person where ";
        let mut token_list = TokenList::from(sql);
        token_list.group();
        let id = token_list.token_idx(Some(6)).unwrap();
        let real_name = id.get_real_name();
        let parent_name = id.get_parent_name();
        let alias = id.get_alias();
        assert_eq!(real_name, Some("person"));
        assert_eq!(parent_name, Some("test"));
        assert_eq!(alias, None);

        let sql = "select * from person where ";
        let mut token_list = TokenList::from(sql);
        token_list.group();
        let id = token_list.token_idx(Some(6)).unwrap();
        let real_name = id.get_real_name();
        let parent_name = id.get_parent_name();
        assert_eq!(real_name, Some("person"));
        assert_eq!(parent_name, None);
    }

    #[test]
    fn test_group_operator() {
        let sqls = vec!["foo+100", "foo + 100", "foo*100"];
        for sql in sqls {
            let mut token_list = TokenList::from(sql);
            token_list.group();
            assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Operation);
        }
    }

    #[test]
    fn test_grouping_where() {
        let sql = "select * from foo where bar = 1 order by id desc";
        let mut token_list = TokenList::from(sql);
        token_list.group();
        assert_eq!(token_list.len(), 12);
    }
}