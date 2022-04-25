use regex::Regex;

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
}