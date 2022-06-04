use sqlparse::{TokenType, group_tokenlist};
use sqlparse::{FormatOption, format};

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


#[test]
fn test_reg26() {
    let sqls = vec!["--hello", "-- hello", "--hello\n", "--", "--\n"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.len(), 1);
        assert_eq!(token_list.tokens[0].typ, TokenType::CommentSingle);
    }
}


#[test]
fn test_reg34() {
    let sql = "create";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::KeywordDDL)
}

#[test]
fn test_reg35() {
    let sql = "select * from foo where bar = 1 limit 1";
    let mut formatter = FormatOption::default_reindent();
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, vec![
        "select *",
        "from foo",
        "where bar = 1",
        "limit 1",
    ].join("\n"));
}

#[test]
fn test_reg39() {
    let sql = "select user.id from user";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 7);
    assert_eq!(token_list.tokens[2].typ, TokenType::Identifier);
    let token_list = &token_list.tokens[2].children;
    assert_eq!(token_list.tokens[0].value, "user");
    assert_eq!(token_list.tokens[1].typ, TokenType::Punctuation);
    assert_eq!(token_list.tokens[2].value, "id");
}


#[test]
fn test_reg40() {
    let sql = "SELECT id, name FROM (SELECT id, name FROM bar) as foo";
    let token_list = group_tokenlist(sql);
    // println!("{}", token_list);
    assert_eq!(token_list.len(), 7);
    assert_eq!(token_list.tokens[2].typ, TokenType::IdentifierList);
    assert_eq!(token_list.tokens.last().unwrap().typ, TokenType::Identifier);
    let token_list = &token_list.tokens.last().unwrap().children;
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.tokens[3].typ, TokenType::IdentifierList);

    let sql = "SELECT id ==  name FROM (SELECT id, name FROM bar)";
    let mut formatter = FormatOption::default_reindent();
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, vec![
        "SELECT id == name",
        "FROM",
        "  (SELECT id,",
        "          name",
        "   FROM bar)",
    ].join("\n"));

    let sql = "SELECT id ==  name FROM (SELECT id, name FROM bar) as foo";
    let mut formatter = FormatOption::default_reindent();
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, vec![
        "SELECT id == name",
        "FROM",
        "  (SELECT id,",
        "          name",
        "   FROM bar) as foo",
    ].join("\n"));
}

#[test]
fn test_reg78() {
    let sqls = vec![
        ("select x.y::text as z from foo", "z"),
        (r#"select x.y::text as "z" from foo"#, r#""z""#),
        (r#"select x."y"::text as z from foo"#, "z"),
        (r#"select x."y"::text as "z" from foo"#, r#""z""#),
        (r#"select "x".y::text as z from foo"#, "z"),
        (r#"select "x".y::text as "z" from foo"#, r#""z""#),
        (r#"select "x"."y"::text as z from foo"#, "z"),
        (r#"select "x"."y"::text as "z" from foo"#, r#""z""#)
    ];
    for (sql, name) in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.tokens[2].typ, TokenType::Identifier);
        let token_list = &token_list.tokens[2].children;
        assert_eq!(token_list.tokens.last().unwrap().value,  name);
    }
}

#[test]
fn test_reg_dont_alias_keywords() {
    let sql = "FROM AS foo";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 5);
    assert_eq!(token_list.tokens[0].typ, TokenType::Keyword);
    assert_eq!(token_list.tokens[2].typ, TokenType::Keyword);
}

#[test]
fn test_reg90() {
    let sql = r#"UPDATE "gallery_photo" SET "owner_id" = 4018, "deleted_at" = NULL, "width" = NULL, "height" = NULL, "rating_votes" = 0, "rating_score" = 0, "thumbnail_width" = NULL, "thumbnail_height" = NULL, "price" = 1, "description" = NULL"#;
    let mut formatter = FormatOption::default_reindent();
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, vec![
        r#"UPDATE "gallery_photo""#,
        r#"SET "owner_id" = 4018,"#,
        r#"    "deleted_at" = NULL,"#,
        r#"    "width" = NULL,"#,
        r#"    "height" = NULL,"#,
        r#"    "rating_votes" = 0,"#,
        r#"    "rating_score" = 0,"#,
        r#"    "thumbnail_width" = NULL,"#,
        r#"    "thumbnail_height" = NULL,"#,
        r#"    "price" = 1,"#,
        r#"    "description" = NULL"#,
    ].join("\n"));
}

#[test]
fn test_reg_except_formatting() {
    let sql= "SELECT 1 FROM foo WHERE 2 = 3 EXCEPT SELECT 2 FROM bar WHERE 1 = 2";
    let mut formatter = FormatOption::default_reindent();
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, vec![
        "SELECT 1",
        "FROM foo",
        "WHERE 2 = 3",
        "EXCEPT",
        "SELECT 2",
        "FROM bar",
        "WHERE 1 = 2",
    ].join("\n")); 
}

#[test]
fn test_reg_null_with_as() {
    let sql = "SELECT NULL AS c1, NULL AS c2 FROM t1";
    let mut formatter = FormatOption::default_reindent();
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, vec![
        "SELECT NULL AS c1,",
        "       NULL AS c2",
        "FROM t1",
    ].join("\n")); 
}

#[test]
fn test_reg213_leadingws() {
    let sql = " select * from foo";
    let mut formatter = FormatOption::default();
    formatter.strip_whitespace = true;
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, "select * from foo");
}


#[test]
fn test_reg207_runaway_format() {
    let sql = "select 1 from (select 1 as one, 2 as two, 3 from dual) t0";
    let mut formatter = FormatOption::default_reindent();
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, vec![
        "select 1",
        "from",
        "  (select 1 as one,",
        "          2 as two,",
        "          3",
        "   from dual) t0",
    ].join("\n")); 
}

// TODO: test_token_next_doesnt_ignore_skip_cm

#[test]
fn test_reg322_concurrently_is_keyword() {
    let sql = "CREATE INDEX CONCURRENTLY myindex ON mytable(col1);";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 12);
    assert_eq!(token_list.tokens[0].typ, TokenType::KeywordDDL);
    assert_eq!(token_list.tokens[2].typ, TokenType::Keyword);
    assert_eq!(token_list.tokens[4].typ, TokenType::Keyword);
    assert_eq!(token_list.tokens[4].value, "CONCURRENTLY");
    assert_eq!(token_list.tokens[6].typ, TokenType::Identifier);
    assert_eq!(token_list.tokens[6].value, "myindex");
}

// TODO: test_issue484_comments_and_newlines

// FIXME: test_reg489_tzcasts

#[test]
fn test_reg_as_in_parentheses_indents() {
    let sql = "(as foo)";
    let mut formatter = FormatOption::default_reindent();
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, "(as foo)");
}

#[test]
fn test_reg_format_invalid_where_clause() {
    let sql = "where, foo";
    let mut formatter = FormatOption::default_reindent();
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, "where, foo");
}