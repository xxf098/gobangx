use std::cmp::Ordering;
use regex::Regex;


pub fn sort_by_length_desc(strs: &mut Vec<&str>) {
    strs.sort_by(|a, b| {
        let o = b.len().cmp(&a.len());
        if o == Ordering::Equal { a.cmp(b) } else { o }
    });
}

#[inline]
pub fn escape_reg_exp(text: &str) -> String {
    regex::escape(text)
}

pub fn create_operator_regex(mut operators: Vec<&str>) -> anyhow::Result<Regex> {
    sort_by_length_desc(&mut operators);
    let s = format!(r"^({}|.)", operators.iter().map(|s| escape_reg_exp(&s)).collect::<Vec<_>>().join("|"));
    let reg = Regex::new(&s)?;
    Ok(reg)
}

pub fn create_line_comment_regex(line_comment_types: Vec<&str>) -> anyhow::Result<Regex> {
    let l = line_comment_types.iter().map(|s| escape_reg_exp(&s)).collect::<Vec<_>>().join("|");
    let s = format!(r"^((?:{}).*?)(?:\r\n|\r|\n|$)", l); 
    let reg = Regex::new(&s)?;
    Ok(reg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_by_length_desc() {
        let mut strs = vec!["aaa", "bb", "ccccc", "ddd", "eeeeeeee", "ff", "h", "g"];
        sort_by_length_desc(&mut strs);
        assert_eq!(strs, vec!["eeeeeeee", "ccccc", "aaa", "ddd", "bb", "ff", "g", "h"])
    }

    #[test]
    fn test_escape_reg_exp() {
        let s = escape_reg_exp("?");
        assert_eq!(s, r"\?");
    }

    #[test]
    fn test_create_line_comment_regex() {
        let line_comment_types = vec!["--"];
        let reg = create_line_comment_regex(line_comment_types).unwrap();
        assert_eq!(reg.as_str(), r"^((?:\-\-).*?)(?:\r\n|\r|\n|$)");
    }

    #[test]
    fn test_create_operator_regex() {
        let operators = vec!["<>", "<=", ">="];
        let reg = create_operator_regex(operators).unwrap();
        assert_eq!(reg.as_str(), "^(<=|<>|>=|.)");
    }
    
}
