use regex::Regex;
use sqlparse::{parse_no_grouping};
use sqlparse::lexer::Token;

const logical_operators: [&str; 4] = ["AND", "OR", "NOT", "BETWEEN"];

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

fn find_prev_keyword(sql: &str) -> Option<(Token, String)>{
    let parsed = parse_no_grouping(sql);
    let pos = parsed.iter().rposition(|t| {
        t.value == "(" || (t.is_keyword() && logical_operators.iter().find(|l| **l == t.normalized).is_none())
    });
    if pos.is_none() {
        return None
    }
    let pos = pos.unwrap();
    let token = parsed[pos].clone();
    let sub_sql = parsed[..pos+1].iter().map(|t| t.value.as_ref()).collect::<Vec<_>>().join("");
    return Some((token, sub_sql))
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
        println!("{:?}", prev)
    }
}