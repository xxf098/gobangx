use regex::Regex;
use sqlparse::{Token, TokenType, Parser};
use super::SuggestTable;

const LOGICAL_OPERATORS: [&str; 4] = ["AND", "OR", "NOT", "BETWEEN"];

pub fn last_word<'a>(text: &'a str, include: &str) -> &'a str {
    if text.len() < 1 {
        return ""
    }
    if text.chars().last().map(|c| c.is_whitespace()).unwrap_or(false) {
        return ""
    }
    let reg =  match include {
        "many_punctuations" => Regex::new(r"([^():,\s]+)$").unwrap(),
        "most_punctuations" => Regex::new(r"([^\.():,\s]+)$").unwrap(),
        "all_punctuations" => Regex::new(r"([^\s]+)$").unwrap(),
        _ => Regex::new(r"(\w+)$").unwrap(),
    };

    if let Some(caps) = reg.captures(text) {
        caps.get(1).map_or("", |m| m.as_str())
    } else {
        ""
    }
}

fn is_keyword_no_grouping(t: &Token) -> bool {
    t.is_keyword() || t.typ == TokenType::KeywordDML || t.typ == TokenType::KeywordRaw
}

pub fn find_prev_keyword(sql: &str, parser: &Parser) -> (Option<Token>, String) {
    let parsed = parser.parse_no_grouping(sql);
    // println!("parsed {:?}", parsed);
    let pos = parsed.iter().rposition(|t| {
        t.value == "(" || (is_keyword_no_grouping(t) && LOGICAL_OPERATORS.iter().find(|l| **l == t.normalized).is_none())
    });
    if pos.is_none() {
        return (None, "".to_string())
    }
    let pos = pos.unwrap();
    let token = parsed[pos].clone();
    let sub_sql = parsed[..pos+1].iter().map(|t| t.value.as_ref()).collect::<Vec<_>>().join("");
    return (Some(token), sub_sql)
}

pub fn extract_tables(sql: &str, parser: &Parser) -> Vec<SuggestTable> {
    let parsed = parser.parse(sql);
    if parsed.len() < 1 {
        return vec![]
    }
    let insert_stmt = parsed[0].value.to_lowercase() == "insert";
    let tokens = extract_from_part(parsed, insert_stmt);
    extract_table_identifiers(tokens)
}

fn is_subselect(token: &Token) -> bool {
    if !token.is_group() {
        return false
    }
    for item in token.children.tokens.iter() {
        if item.typ == TokenType::DML && ["SELECT", "INSERT", "UPDATE", "CREATE", "DELETE"].iter().find(|t| **t == item.value.to_uppercase()).is_some() {
            return true
        }
    }
    false
}

fn extract_from_part(parsed: Vec<Token>, stop_at_punctuation: bool) -> Vec<Token> {
    let mut tbl_prefix_seen = false;
    let mut tokens = vec![];
    for item in parsed {
        if tbl_prefix_seen {
            if is_subselect(&item) {
                let sub_tokens = extract_from_part(item.children.tokens, stop_at_punctuation);
                tokens.extend(sub_tokens);
            } else if stop_at_punctuation && item.typ == TokenType::Punctuation {
                break
            } else if item.typ == TokenType::Keyword && item.normalized == "ON" {
                tbl_prefix_seen = false;
                continue
            } else if item.typ == TokenType::Keyword && item.normalized != "FROM" && !item.normalized.ends_with("JOIN") {
                break
            } else {
                tokens.push(item);
            }
        } else if (item.is_keyword() || item.typ == TokenType::DML) 
            && ["COPY", "FROM", "INTO", "UPDATE", "TABLE", "JOIN"].iter().find(|t| **t == item.value.to_uppercase()).is_some() {
            tbl_prefix_seen = true;
        } else if item.typ == TokenType::IdentifierList {
            // TODO:
            tbl_prefix_seen = true;
            // break
        }
    }
    tokens
}

fn extract_table_identifiers(tokens: Vec<Token>) -> Vec<SuggestTable> {
    let mut tables = vec![];
    for item in tokens {
        if item.typ == TokenType::IdentifierList {
            let identifiers = item.children.get_identifiers();
            for id in identifiers {
                let t = item.children.token_idx(Some(id)).unwrap();
                let schema_name = t.get_parent_name();
                if let Some(real_name) = t.get_real_name() {
                    tables.push(SuggestTable::new(schema_name, real_name, t.get_alias()));
                }
            }
            continue
        } else if item.typ == TokenType::Identifier {
            let real_name = item.get_real_name();
            let schema_name = item.get_parent_name();
            if real_name.is_some() {
                let alias = item.get_alias();
                tables.push(SuggestTable::new(schema_name, real_name.unwrap(), alias));
            } else {
                let name = item.get_name();
                let alias = item.get_alias().or(item.get_name());
                tables.push(SuggestTable::new(None, name.unwrap(), alias))
            }
        } else if item.typ == TokenType::Function {
            continue
        }
    }
    return tables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_word() {
        let mut text = "abc ";
        let mut w = last_word(text, "");
        assert_eq!(w, "");
        text = "abc def";
        w = last_word(text, "");
        assert_eq!(w, "def");
        text = "abc def;";
        w = last_word(text, "");
        assert_eq!(w, "");
        text = "bac $def";
        w = last_word(text, "");
        assert_eq!(w, "def");
        text = "bac $def";
        w = last_word(text, "many_punctuations");
        assert_eq!(w, "$def");
    }

    #[test]
    fn test_find_prev_keyword() {
        let sql = "select * from users where id > 0";
        let p = Parser::default();
        let prev = find_prev_keyword(sql, &p);
        assert!(prev.0.is_some());
        assert_eq!(prev.0.map(|p| p.value).unwrap(), "where");
        // println!("{:?}", prev)
    }

    #[test]
    fn test_find_prev_keyword1() {
        let sql = "select id,";
        let p = Parser::default();
        let prev = find_prev_keyword(sql, &p);
        assert!(prev.0.is_some());
        assert_eq!(prev.0.map(|p| p.value).unwrap(), "select");
    }

    #[test]
    fn test_extract_tables() {
        let sql = "select * from test.person where ";
        let p = Parser::default();
        let suggestions = extract_tables(sql, &p);
        assert_eq!(suggestions.len(), 1);
        let suggestion = &suggestions[0];
        assert_eq!(suggestion.schema.as_deref(), Some("test"));
        assert_eq!(suggestion.table, "person".to_string());
        assert_eq!(suggestion.alias, None);
    }

    #[test]
    fn test_extract_tables1() {
        let sql = "SELECT MAX(col1 +  FROM tbl'";
        let p = Parser::default();
        let suggestions = extract_tables(sql, &p);
        assert_eq!(suggestions.len(), 1);
        let suggestion = &suggestions[0];
        assert_eq!(suggestion.schema, None);
        assert_eq!(suggestion.table, "tbl".to_string());
        assert_eq!(suggestion.alias, None);
    }

    #[test]
    fn test_extract_tables_alias() {
        let sql = "SELECT t1. FROM tabl1 t1, tabl2 t2";
        let p = Parser::default();
        let suggestions = extract_tables(sql, &p);
        assert_eq!(suggestions[0], SuggestTable::new(None, "tabl1", Some("t1")));
        assert_eq!(suggestions[1], SuggestTable::new(None, "tabl2", Some("t2")));
    }

    #[test]
    fn test_extract_tables_sub_query() {
        let sql = "SELECT * FROM (SELECT  FROM abc";
        let p = Parser::default();
        let suggestions = extract_tables(sql, &p);
        assert_eq!(suggestions[0], SuggestTable::new(None, "abc", None));
    }
}