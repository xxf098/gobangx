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