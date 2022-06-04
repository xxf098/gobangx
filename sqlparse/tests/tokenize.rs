use sqlparse::{TokenType, parse_no_grouping};

#[test]
fn test_tokenize_simple() {
    let sql = "select * from foo;";
    let tokens = parse_no_grouping(sql);
    assert_eq!(tokens.len(), 8);
    assert_eq!(tokens[0].typ, TokenType::KeywordDML);
    assert_eq!(tokens[7].typ, TokenType::Punctuation);
    assert_eq!(tokens[7].value, ";");
}

#[test]
fn test_tokenize_backticks() {
    let sql = "`foo`.`bar`";
    let tokens = parse_no_grouping(sql);
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].typ, TokenType::Name);
    assert_eq!(tokens[0].value, "`foo`");
}

#[test]
fn test_tokenize_linebreaks() {
    let sqls = vec!["foo\nbar\n", "foo\rbar\r", "foo\r\nbar\r\n", "foo\r\nbar\n"];
    for sql in sqls {
        let tokens = parse_no_grouping(sql);
        let s = tokens.iter().map(|t| t.value.clone()).collect::<Vec<_>>().join("");
        assert_eq!(sql, s);
    }
}


#[test]
fn test_tokenize_inline_keywords() {
    let sql = "create created_foo";
    let tokens = parse_no_grouping(sql);
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].typ, TokenType::KeywordDDL);
    assert_eq!(tokens[2].typ, TokenType::Name);
    assert_eq!(tokens[2].value, "created_foo");
    
    let sql = "enddate";
    let tokens = parse_no_grouping(sql);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].typ, TokenType::Name);

    let sql = "join_col";
    let tokens = parse_no_grouping(sql);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].typ, TokenType::Name);

    let sql = "left join_col";
    let tokens = parse_no_grouping(sql);
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[2].typ, TokenType::Name);
    assert_eq!(tokens[2].value, "join_col");
}

#[test]
fn test_tokenize_negative_numbers() {
    let sql = "values(-1)";
    let tokens = parse_no_grouping(sql);
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[2].typ, TokenType::NumberInteger);
    assert_eq!(tokens[2].value, "-1");
}

#[test]
fn test_parse_join() {
    let sqls = vec![
        "JOIN foo",
        "LEFT JOIN foo",
        "LEFT OUTER JOIN foo",
        "FULL OUTER JOIN foo",
        "NATURAL JOIN foo",
        "CROSS JOIN foo",
        "STRAIGHT JOIN foo",
        "INNER JOIN foo",
        "LEFT INNER JOIN foo",
    ];
    for sql in sqls {
        let tokens = parse_no_grouping(sql);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].typ, TokenType::Keyword)
    }
}

#[test]
fn test_parse_union() {
    unimplemented!();
}