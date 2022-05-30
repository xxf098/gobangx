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
    let sqls = vec![
        ("select a from b where c < d + e", TokenType::Identifier, TokenType::Identifier),
        // ("select a from b where c < d + interval 1 day", TokenType::Identifier, TokenType::TypedLiteral)
    ];
    for (sql, a, b) in sqls {
        let token_list = group_tokenlist(sql);
        // println!("{}", token_list);
        assert_eq!(token_list.len(), 9);
        assert_eq!(token_list.tokens[2].typ, TokenType::Identifier);
        assert_eq!(token_list.tokens[6].typ, TokenType::Identifier);
        assert_eq!(token_list.tokens[8].typ, TokenType::Where);
        let where_token = &token_list.tokens[8].children;
        assert_eq!(where_token.tokens[2].typ, TokenType::Comparison);
        assert_eq!(where_token.tokens.len(), 3);
        let comparison = &where_token.tokens[2].children;
        assert_eq!(comparison.tokens[0].typ, TokenType::Identifier);
        assert_eq!(comparison.tokens[2].typ, TokenType::OperatorComparison);
        assert_eq!(comparison.tokens[4].typ, TokenType::Operation);
        assert_eq!(comparison.len(), 5);
        let operation = &comparison.tokens[4].children;
        assert_eq!(operation.tokens[0].typ, a);
        assert_eq!(operation.tokens[2].typ, TokenType::Operator);
        assert_eq!(operation.tokens[4].typ, b);
        assert_eq!(operation.len(), 5);
    }
}