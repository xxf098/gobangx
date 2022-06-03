use sqlparse::{TokenType, group_tokenlist};

#[test]
fn test_reg1() {
    // make sure where doesn't consume parenthesis
    let sql = "(where 1)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Parenthesis);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.len(), 3);
    assert_eq!(token_list.tokens[0].typ, TokenType::Punctuation);
    assert_eq!(token_list.tokens[token_list.len()-1].typ, TokenType::Punctuation);
}

