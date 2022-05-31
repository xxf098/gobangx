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


#[test]
fn test_grouping_identifiers() {
    let sql = r#"select foo.bar from "myscheme"."table" where fail. order"#;
    let token_list = group_tokenlist(sql);
    // println!("{}", token_list);
    assert_eq!(token_list.tokens[2].typ, TokenType::Identifier);
    assert_eq!(token_list.tokens[6].typ, TokenType::Identifier);
    assert_eq!(token_list.tokens[8].typ, TokenType::Where);
    let sql = "select * from foo where foo.id = 1";
    let token_list = group_tokenlist(sql);
    let n = token_list.tokens.len();
    let children = &token_list.tokens[n-1].children;
    let t = &children.tokens[children.len()-1].children.tokens[0];
    assert_eq!(t.typ, TokenType::Identifier);
    let sql = r#"select * from (select "foo"."id" from foo)"#;
    let token_list = group_tokenlist(sql);
    let c = &token_list.tokens[token_list.len()-1].children;
    assert_eq!(c.tokens[3].typ, TokenType::Identifier);

    let sql = "select 1.0*(a+b) as col, sum(c)/sum(d) from myschema.mytable";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 7);
    assert_eq!(token_list.tokens[2].typ, TokenType::IdentifierList);
    assert_eq!(token_list.tokens[2].children.len(), 4);
    let identifier_list = &token_list.tokens[2].children;
    let identifiers = identifier_list.get_identifiers();
    assert_eq!(identifiers.len(), 2);
    assert_eq!(identifier_list.tokens[identifiers[0]].get_alias(), Some("col"));
}


#[test]
fn test_grouping_simple_identifiers() {
    let sqls = vec!["1 as f", "foo as f", "foo f", "1/2 as f", "1/2 f", "1<2 as f", "1<2 f"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.tokens[0].typ, TokenType::Identifier);
    }
}

#[test]
fn test_grouping_identifier_list() {
    let sqls = vec![
        "foo, bar",
        "sum(a), sum(b)",
        "sum(a) as x, b as y",
        "sum(a)::integer, b",
        "sum(a)/count(b) as x, y",
        "sum(a)::integer as x, y",
        "sum(a)::integer/count(b) as x, y",
        ];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.tokens[0].typ, TokenType::IdentifierList);
    }
    
}

#[test]
fn test_grouping_identifier_wildcard() {
    let sql = "a.*, b.id";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::IdentifierList);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.tokens[0].typ, TokenType::Identifier);
    assert_eq!(token_list.tokens[token_list.len()-1].typ, TokenType::Identifier);
}

#[test]
fn test_grouping_identifier_name_wildcard() {
    let sql = "a.*";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].children.len(), 3);
}


#[test]
fn test_grouping_identifier_invalid_in_middle() {
    let sql = "SELECT foo. FROM foo";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[2].typ, TokenType::Identifier);
    assert_eq!(token_list.tokens[2].children.tokens[1].typ, TokenType::Punctuation);
    assert_eq!(token_list.tokens[3].typ, TokenType::Whitespace);
    assert_eq!(token_list.tokens[2].value, "foo.");
}


#[test]
fn test_grouping_identifer_as() {
    let sqls = vec!["foo as (select *)", "foo as(select *)"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        // println!("{}", token_list);
        assert_eq!(token_list.tokens[0].typ, TokenType::Identifier);
        let token = &token_list.tokens[0].children.tokens[2];
        assert_eq!(token.typ, TokenType::Keyword);
        assert_eq!(token.normalized, "AS");
    }
}

// #[test]
// fn test_grouping_identifier_as_invalid() {
//     let sql = "foo as select *";
//     let token_list = group_tokenlist(sql);
//     println!("{}", token_list);
// }

#[test]
fn test_grouping_identifier_function() {
    let sql = "foo() as bar";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::Identifier);
    assert_eq!(token_list.tokens[0].children.tokens[0].typ, TokenType::Function);

    let sql = "foo()||col2 bar";
    let token_list = group_tokenlist(sql);
    // println!("{}", token_list);
    assert_eq!(token_list.tokens[0].typ, TokenType::Identifier);
    assert_eq!(token_list.tokens[0].children.tokens[0].typ, TokenType::Operation);
    assert_eq!(token_list.tokens[0].children.tokens[0].children.tokens[0].typ, TokenType::Function);
}

#[test]
fn test_grouping_operation() {
    let sqls = vec!["foo+100", "foo + 100", "foo*100"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Operation);
    }
}


#[test]
fn test_grouping_identifier_list1() {
    let sql = "a, b, c";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::IdentifierList);
    let sql = "(a, b, c)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].children.tokens[1].typ, TokenType::IdentifierList);
}


#[test]
fn test_grouping_identifier_list_subquery() {
    let sql = "select * from (select a, b + c as d from table) sub";
    let token_list = group_tokenlist(sql);
    let subquery = &token_list.tokens[token_list.len()-1].children;
    let token_list = &subquery.tokens[0].children;
    let types = vec![TokenType::IdentifierList];
    let idx = token_list.token_next_by(&types, None, 0);
    assert!(idx.is_some());
    let types = vec![TokenType::Identifier];
    let idx = token_list.token_next_by(&types, None, idx.unwrap());
    assert!(idx.is_none());
}

#[test]
fn test_grouping_identifier_list_case() {
    let sql = "a, case when 1 then 2 else 3 end as b, c";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::IdentifierList);
    let sql = "(a, case when 1 then 2 else 3 end as b, c)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].children.tokens[1].typ, TokenType::IdentifierList);
}

#[test]
fn test_grouping_identifier_list_other() {
    let sql = "select *, null, 1, 'foo', bar from mytable, x";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[2].typ, TokenType::IdentifierList);
    assert_eq!(token_list.tokens[2].children.len(), 13);
}

// TODO:
// test_grouping_identifier_list_with_inline_comments

#[test]
fn test_grouping_identifier_list_with_order() {
    let sql = "1, 2 desc, 3";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::IdentifierList);
    let token = &token_list.tokens[0].children.tokens[3];
    assert_eq!(token.typ, TokenType::Identifier);
    assert_eq!(token.value, "2 desc");
}
