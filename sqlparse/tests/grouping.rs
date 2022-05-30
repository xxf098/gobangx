use sqlparse::{TokenType, group_tokenlist};

#[test]
fn test_grouping_parenthesis() {
    let sql = "select (select (x3) x2) and (y2) bar";

    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 7);
    assert_eq!(token_list.token_idx(Some(2)).unwrap().typ, TokenType::Parenthesis);
    assert_eq!(token_list.token_idx(Some(6)).unwrap().typ, TokenType::Identifier);
    let sub_tokens = &token_list.token_idx(Some(2)).unwrap().children;
    assert_eq!(sub_tokens.token_idx(Some(3)).unwrap().typ, TokenType::Parenthesis);

}


#[test]
fn test_grouping_assignment() {
    let sqls = vec!["foo := 1", "foo := 1;"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.len(), 1);
        assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Assignment);
    }
}

#[test]
fn test_grouping_typed_literal() {
    let sqls = vec!["x > DATE '2020-01-01'", "x > TIMESTAMP '2020-01-01 00:00:00'"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.tokens[0].children.token_idx(Some(4)).unwrap().typ, TokenType::TypedLiteral);
    }
}

#[test]
fn test_compare_expr() {
    
}