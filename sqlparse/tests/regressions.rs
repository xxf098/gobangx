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