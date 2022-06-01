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
