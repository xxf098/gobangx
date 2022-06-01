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

#[test]
fn test_grouping_where() {
    let sql = "select * from foo where bar = 1 order by id desc";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 12);

    let sql = "select x from (select y from foo where bar = 1) z";
    // let sql = "select y from foo where bar = 1";
    let token_list = group_tokenlist(sql);
    let token_list = &token_list.tokens[token_list.len()-1].children.tokens[0].children;
    assert_eq!(token_list.tokens[token_list.len()-2].typ, TokenType::Where);
}

#[test]
fn test_grouping_where_union() {
    let sql = "select 1 where 1 = 2 union select 2";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[5].value, "union");
    let sql = "select 1 where 1 = 2 union all select 2";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[5].value, "union all");
}


#[test]
fn test_returning_kw_ends_where_clause() {
    let sql = "delete from foo where x > y returning z";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[6].typ, TokenType::Where);
    assert_eq!(token_list.tokens[7].typ, TokenType::Keyword);
    assert_eq!(token_list.tokens[7].value, "returning");
}


#[test]
fn test_into_kw_ends_where_clause() {
    let sql = "select * from foo where a = 1 into baz";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[8].typ, TokenType::Where);
    assert_eq!(token_list.tokens[9].typ, TokenType::Keyword);
    assert_eq!(token_list.tokens[9].value, "into");
}

#[test]
fn test_grouping_typecast() {
    let sqls = vec![
        ("select foo::integer from bar", "integer"),
        ("select (current_database())::information_schema.sql_identifier", "information_schema.sql_identifier"),
        ];
    for (sql, value) in sqls {
        let token_list = group_tokenlist(sql);
        let token_list = &token_list.tokens[2].children;
        assert_eq!(token_list.tokens[token_list.len()-1].value, value);
    }
}

#[test]
fn test_grouping_alias() {
    let sql = "select foo as bar from mytable";
    let token_list = group_tokenlist(sql);
    let token_list = &token_list.tokens[2].children;
    assert_eq!(token_list.tokens[0].value, "foo");
    assert_eq!(token_list.tokens[token_list.len()-1].value, "bar");

    let sql = "select foo from mytable t1";
    let token_list = group_tokenlist(sql);
    let token_list = &token_list.tokens[6].children;
    assert_eq!(token_list.tokens[0].value, "mytable");
    assert_eq!(token_list.tokens[token_list.len()-1].value, "t1");

    let sql = "select foo::integer as bar from mytable";
    let token_list = group_tokenlist(sql);
    let token_list = &token_list.tokens[2].children;
    assert_eq!(token_list.tokens[token_list.len()-1].value, "bar");

    let sql = "SELECT DISTINCT (current_database())::information_schema.sql_identifier AS view";
    let token_list = group_tokenlist(sql);
    let token_list = &token_list.tokens[4].children;
    assert_eq!(token_list.tokens[token_list.len()-1].value, "view");
}

#[test]
fn test_grouping_alias_case() {
    let sql = "CASE WHEN 1 THEN 2 ELSE 3 END foo";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.tokens[token_list.len()-1].value, "foo");
}

#[test]
fn test_grouping_subquery_no_parens() {
    let sql = "CASE WHEN 1 THEN select 2 where foo = 1 end";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Case);
}

#[test]
fn test_grouping_idlist_function() {
    let sql = "foo(1) x, bar";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::IdentifierList);
}

#[test]
fn test_grouping_comparison_exclude() {
    // TODO: FIXME
    let sql = "(=)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::Parenthesis);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.tokens[1].typ, TokenType::OperatorComparison);

    let sql = "(a=1)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::Parenthesis);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.tokens[1].typ, TokenType::Comparison);

    let sql = "(a>=1)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::Parenthesis);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.tokens[1].typ, TokenType::Comparison);
}


#[test]
fn test_grouping_function() {
    let sql = "foo()";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Function);
    let sql = "foo(null, bar)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Function);
}

#[test]
fn test_grouping_function_not_in() {
    let sql = "in(1, 2)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 2);
    assert_eq!(token_list.tokens[0].typ, TokenType::OperatorComparison);
    assert_eq!(token_list.tokens[1].typ, TokenType::Parenthesis);
}

#[test]
fn test_grouping_in_comparison() {
    let sql = "a in (1, 2)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.len(), 5);
    assert_eq!(token_list.tokens[0].value, "a");
    assert_eq!(token_list.tokens[token_list.len()-1].value, "(1, 2)");
}


#[test]
fn test_grouping_varchar() {
    let sql = r#""text" Varchar(50) NOT NULL"#;
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[2].typ, TokenType::Function);
}

#[test]
fn test_grouping_identifier_with_operators() {
    let sql = "foo||bar";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Operation);

    let sql = "foo || bar";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Operation);
}

#[test]
fn test_grouping_identifier_with_op_trailing_ws() {
    let sql = "foo || bar ";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 2);
    assert_eq!(token_list.tokens[0].typ, TokenType::Operation);
    assert_eq!(token_list.tokens[1].typ, TokenType::Whitespace);
}

#[test]
fn test_grouping_identifier_with_string_literals() {
    let sql = "foo + 'bar'";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Operation);
}

#[test]
fn test_grouping_identifier_consumes_ordering() {
    let sql = "select * from foo order by c1 desc, c2, c3";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[token_list.len()-1].typ, TokenType::IdentifierList);
    let token_list = &token_list.tokens[token_list.len()-1].children;
    let ids = token_list.get_identifiers();
    assert_eq!(ids.len(), 3);
    assert_eq!(token_list.tokens[ids[0]].value, "c1 desc");
    assert_eq!(token_list.tokens[ids[1]].value, "c2");
}

#[test]
fn test_grouping_comparison_with_keywords() {
    let sql = "foo = NULL";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Comparison);
    assert_eq!(token_list.tokens[0].children.len(), 5);
    let sql = "foo = null";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Comparison);
}

#[test]
fn test_grouping_comparison_with_floats() {
    let sql = "foo = 25.5";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Comparison);
    assert_eq!(token_list.tokens[0].children.len(), 5);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.tokens[0].value, "foo");
    assert_eq!(token_list.tokens[token_list.len()-1].value, "25.5");
}


#[test]
fn test_grouping_comparison_with_parenthesis() {
    let sql = "(3 + 4) = 7";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Comparison);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.tokens[0].typ, TokenType::Parenthesis);
    assert_eq!(token_list.tokens[token_list.len()-1].typ, TokenType::NumberInteger);
}

#[test]
fn test_grouping_comparison_with_strings() {
    let sqls = vec!["foo = bar", "foo != bar", "foo > bar", "foo > bar", "foo <= bar", "foo >= bar", "foo ~ bar",
    "foo ~~ bar", "foo !~~ bar", "foo LIKE bar", "foo NOT LIKE bar", "foo ILIKE bar", "foo NOT ILIKE bar"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.len(), 1);
        assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Comparison);
        let token_list = &token_list.tokens[0].children;
        assert_eq!(token_list.tokens[token_list.len()-1].value, "bar");
    }
}

// TODO:
// test_like_and_ilike_comparison

#[test]
fn test_grouping_comparison_with_functions() {
    let sql = "foo = DATE(bar.baz)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Comparison);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.len(), 5);
    assert_eq!(token_list.tokens[0].value, "foo");
    assert_eq!(token_list.tokens[token_list.len()-1].value, "DATE(bar.baz)");

    let sql = "DATE(foo.bar) = DATE(bar.baz)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Comparison);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.len(), 5);
    assert_eq!(token_list.tokens[0].value, "DATE(foo.bar)");
    assert_eq!(token_list.tokens[token_list.len()-1].value, "DATE(bar.baz)");

    let sql = "DATE(foo.bar) = bar.baz";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Comparison);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.len(), 5);
    assert_eq!(token_list.tokens[0].value, "DATE(foo.bar)");
    assert_eq!(token_list.tokens[token_list.len()-1].value, "bar.baz");
}

#[test]
fn test_grouping_comparison_with_typed_literal() {
    let sql = "foo = DATE 'bar.baz'";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Comparison);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.len(), 5);
    assert_eq!(token_list.tokens[0].value, "foo");
    assert_eq!(token_list.tokens[token_list.len()-1].typ, TokenType::TypedLiteral);
    assert_eq!(token_list.tokens[token_list.len()-1].value, "DATE 'bar.baz'");
}

#[test]
fn test_grouping_forloops() {
    let sqls = vec!["for foo in bar LOOP foobar END LOOP", "FOREACH foo in bar LOOP foobar END LOOP"];
    for sql in sqls.into_iter() {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.len(), 1);
        assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::For);
    }
}

#[test]
fn test_grouping_nested_for() {
    let sql = "FOR foo LOOP FOR bar LOOP END LOOP END LOOP";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    let children_len = token_list.tokens[0].children.len();
    assert_eq!(token_list.tokens[0].children.token_idx(Some(0)).unwrap().normalized, "FOR");
    assert_eq!(token_list.tokens[0].children.token_idx(Some(children_len-1)).unwrap().normalized, "END LOOP");
    let inner = token_list.tokens[0].children.token_idx(Some(6)).unwrap();
    assert_eq!(inner.children.token_idx(Some(0)).unwrap().normalized, "FOR");
    let inner_len = inner.children.len();
    assert_eq!(inner.children.token_idx(Some(inner_len-1)).unwrap().normalized, "END LOOP");
}

#[test]
fn test_grouping_begin() {
    let sql = "BEGIN foo END";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.token_idx(Some(0)).unwrap().typ, TokenType::Begin);
}

#[test]
fn test_nested_begin() {
    let sql = "BEGIN foo BEGIN bar END END";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    let children_len = token_list.tokens[0].children.len();
    assert_eq!(token_list.tokens[0].children.token_idx(Some(0)).unwrap().normalized, "BEGIN");
    assert_eq!(token_list.tokens[0].children.token_idx(Some(children_len-1)).unwrap().normalized, "END");
    let inner = token_list.tokens[0].children.token_idx(Some(4)).unwrap();
    assert_eq!(inner.children.token_idx(Some(0)).unwrap().normalized, "BEGIN");
    let inner_len = inner.children.len();
    assert_eq!(inner.children.token_idx(Some(inner_len-1)).unwrap().normalized, "END");
}