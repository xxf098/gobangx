use crate::sql::parse::lexer::{Token, TokenList};
use crate::sql::parse::tokens::TokenType;

pub fn group(stmt: Vec<Token>) -> Vec<Token> {
    // stmt.into_iter().map(|s| s.into()).collect::<Vec<_>>()
    vec![]
}


pub fn group_identifier(mut tokens: TokenList) -> TokenList {
    let ttypes = vec![TokenType::StringSymbol, TokenType::Name];
    let mut tidx = tokens.token_next_by(&ttypes, 0);
    while let Some(idx) = tidx {
        tokens.group_tokens(TokenType::Identifier, idx, idx +1);
        tidx = tokens.token_next_by(&ttypes, idx+1);
    }
    tokens
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parse::lexer::{tokenize};

    #[test]
    fn test_group_identifier() {
        let sql = "select * from users;";
        let tokens = tokenize(sql);
        let tokens = TokenList::new(tokens);
        let tokens = group_identifier(tokens);
        println!("{:?}", tokens.tokens);
    }
}