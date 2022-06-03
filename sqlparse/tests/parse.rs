use sqlparse::{TokenType, group_tokenlist};

#[test]
fn test_parse_float() {
    let sqls = vec![".5", ".51", "1.5", "12.5"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.len(), 1);
        assert_eq!(token_list.tokens[0].typ, TokenType::NumberFloat);      
    }
}

#[test]
fn test_parse_placeholder() {
    let sqls = vec![
        ("select * from foo where user = ?", "?"),
        ("select * from foo where user = :1", ":1"),
        ("select * from foo where user = :name", ":name"),
        ("select * from foo where user = %s", "%s"),
        ("select * from foo where user = $a", "$a"),
        ];
    for (sql, placeholder) in sqls {
        let token_list = group_tokenlist(sql);
        let token_list = &token_list.tokens[token_list.len()-1].children;
        assert_eq!(token_list.tokens.last().unwrap().typ, TokenType::NamePlaceholder);
        assert_eq!(token_list.tokens.last().unwrap().value, placeholder);
    }
}


#[test]
fn test_parse_modulo_not_placeholder() {
    let sql = "x %3";
    let token_list = group_tokenlist(sql);
    let token_list = &token_list.tokens[0].children;
    assert_eq!(token_list.tokens[2].typ, TokenType::Operator);
}

#[test]
fn test_parse_access_symbol() {
    // FIXME: Square Bracket
    let sql = "select a.[foo bar] as foo";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens.last().unwrap().typ, TokenType::Identifier);
    let token_list = &token_list.tokens.last().unwrap().children;
    assert_eq!(token_list.tokens[token_list.len()-1].value, "foo");
    assert_eq!(token_list.tokens[0].value, "a");
    assert_eq!(token_list.tokens[2].value, "[foo bar]");
}

#[test]
fn test_parse_square_brackets_notation_isnt_too_greedy() {
    let sql = "[foo], [bar]";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 4);
    assert_eq!(token_list.tokens[0].value, "[foo]");
    assert_eq!(token_list.tokens[token_list.len()-1].value, "[bar]");
}

#[test]
fn test_parse_square_brackets_notation_isnt_too_greedy2() {
    let sql = "[(foo[i])]";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[0].typ, TokenType::SquareBrackets);
}

#[test]
fn test_parse_keyword_like_identifier() {
    let sql = "foo.key";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Identifier);
}

#[test]
fn test_parse_function_parameter() {
    let sql = "abs(some_col)";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].children.tokens[0].typ, TokenType::Identifier);
    
}

#[test]
fn test_parse_function_param_single_literal() {
    let sql = "foo(5)";
    let token_list = group_tokenlist(sql);
    let token_list = &token_list.tokens[0].children.tokens[1].children;
    assert_eq!(token_list.tokens[token_list.len()-2].typ, TokenType::NumberInteger);
}

#[test]
fn test_parse_nested_function() {
    let sql = "foo(bar(5))";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
}

#[test]
fn test_parse_quoted_identifier() {
    let sql = r#"select x.y as "z" from foo'"#;
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.tokens[2].typ, TokenType::Identifier);
}

#[test]
fn test_parse_valid_identifier_names() {
    let sqls = vec!["name", "foo", "_foo", "1_data"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.tokens[0].typ, TokenType::Identifier);
    }
}

// test_psql_quotation_marks

#[test]
fn test_parse_double_precision_is_builtin() {
    let sql = "DOUBLE PRECISION";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].value, "DOUBLE PRECISION");
    assert_eq!(token_list.tokens[0].typ, TokenType::NameBuiltin);
}

#[test]
fn test_parse_placeholder1() {
    let sql = "?";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::NamePlaceholder);
}


#[test]
fn test_parse_scientific_numbers() {
    let sqls = vec!["6.67428E-8", "1.988e33", "1e-12"];
    for sql in sqls {
        let token_list = group_tokenlist(sql);
        assert_eq!(token_list.tokens[0].typ, TokenType::NumberFloat);
    }
}

#[test]
fn test_parse_single_quotes_are_strings() {
    let sql = "'foo'";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::StringSingle);
}

#[test]
fn test_parse_double_quotes_are_identifiers() {
    let sql = r#""foo""#;
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::Identifier);
}


#[test]
fn test_parse_single_quotes_with_linebreaks() {
    let sql = "'f\nf'";
    let token_list = group_tokenlist(sql);
    assert_eq!(token_list.len(), 1);
    assert_eq!(token_list.tokens[0].typ, TokenType::StringSingle);
}

