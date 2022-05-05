use itertools::Itertools;
use regex::Regex;
use sqlparse::{Token, TokenType, parse, parse_no_grouping};
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

pub fn find_prev_keyword(sql: &str) -> (Option<Token>, String) {
    let parsed = parse_no_grouping(sql);
    let pos = parsed.iter().rposition(|t| {
        t.value == "(" || (t.is_keyword() && LOGICAL_OPERATORS.iter().find(|l| **l == t.normalized).is_none())
    });
    if pos.is_none() {
        return (None, "".to_string())
    }
    let pos = pos.unwrap();
    let token = parsed[pos].clone();
    let sub_sql = parsed[..pos+1].iter().map(|t| t.value.as_ref()).collect::<Vec<_>>().join("");
    return (Some(token), sub_sql)
}

pub fn extract_tables(sql: &str) -> Vec<SuggestTable> {
    let parsed = parse(sql);
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
            } else if item.is_keyword() && item.normalized != "FROM" && !item.normalized.ends_with("JOIN") {
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
            // TODO: 
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
        let prev = find_prev_keyword(sql);
        assert!(prev.0.is_some());
        assert_eq!(prev.0.map(|p| p.value).unwrap(), "where");
        // println!("{:?}", prev)
    }

    #[test]
    fn test_extract_tables() {
        let sql = "select * from test.person where ";
        let suggestions = extract_tables(sql);
        assert_eq!(suggestions.len(), 1);
        let suggestion = &suggestions[0];
        assert_eq!(suggestion.schema.as_deref(), Some("test"));
        assert_eq!(suggestion.table, "person".to_string());
        assert_eq!(suggestion.alias, None);
    }
}