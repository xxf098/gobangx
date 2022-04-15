use regex::Regex;
use super::Token;
use crate::sql::token::regex_factory::{ create_string_regex };

pub struct RegexToken {
    reg: Regex,
    token: Token,
}

impl RegexToken {
    
    fn new(s: &str, token: Token) -> Self {
        Self{
            reg: Regex::new(s).unwrap(), 
            token
        }
    }

    fn new_reg(r: Regex, token: Token) -> Self {
        Self{
            reg: r, 
            token
        }
    }
}

#[inline]
fn new_rt(s: &str, token: Token) -> RegexToken{
    RegexToken::new(s, token)
}


fn sql_regex() -> Vec<RegexToken> {
    let literal = create_string_regex(vec!["''", r#""""#, "``"]).unwrap();
    vec![
        new_rt(r"(--|# )\+.*?(\r\n|\r|\n|$)", Token::CommentSingleHint),
        new_rt(r"/\*\+[\s\S]*?\*/", Token::CommentMultilineHint),

        new_rt(r"(--|# ).*?(\r\n|\r|\n|$)", Token::CommentSingle),
        new_rt(r"/\*[\s\S]*?\*/", Token::CommentMultiline),

        new_rt(r"(\r\n|\r|\n)", Token::Newline),
        new_rt(r"\s+?", Token::Whitespace),

        new_rt(r":=", Token::Assignment),
        new_rt(r"::", Token::Punctuation),

        new_rt(r"\*", Token::Wildcard),

        new_rt(r"`(``|[^`])*`", Token::Name),
        new_rt(r"´(´´|[^´])*´", Token::Name),
        RegexToken::new_reg(literal, Token::Literal),

        new_rt(r"\?", Token::NamePlaceholder),
        new_rt(r"%(\(\w+\))?s", Token::NamePlaceholder),
        // (r'(?<!\w)[$:?]\w+', tokens.Name.Placeholder),

        new_rt(r"\\\w+", Token::Command),
        new_rt(r"(NOT\s+)?(IN)\b", Token::OperatorComparison),

        new_rt(r"(CASE|IN|VALUES|USING|FROM|AS)\b", Token::Keyword),

        new_rt(r"(@|##|#)[A-ZÀ-Ü]\w+", Token::Name),
        new_rt(r"[A-ZÀ-Ü]\w*(?:\s*\.)", Token::Name),
        new_rt(r"(\.)[A-ZÀ-Ü]\w*", Token::Name),
        new_rt(r"[A-ZÀ-Ü]\w*(?:\()", Token::Name),

        new_rt(r"-?0x[\dA-F]+", Token::NumberHexadecimal),
        new_rt(r"-?\d+(\.\d+)?E-?\d+", Token::NumberFloat),
        new_rt(r"-?(\d+(\.\d*)|\.\d+)", Token::NumberFloat),
        new_rt(r"(-\s*)?[0-9]+", Token::NumberInteger),

        new_rt(r"'(''|\\\\|\\'|[^'])*'", Token::StringSingle),
        new_rt(r#""(""|\\\\|\\"|[^"])*""#, Token::StringSymbol),
        new_rt(r#"(""|".*?[^\\]")"#, Token::StringSymbol),
        new_rt(r#"(?:[^\w\])])(\[[^\]\[]+\])"#, Token::Name),

        new_rt(r"((LEFT\s+|RIGHT\s+|FULL\s+)?(INNER\s+|OUTER\s+|STRAIGHT\s+)?|(CROSS\s+|NATURAL\s+)?)?JOIN\b", Token::Keyword),
        new_rt(r"END(\s+IF|\s+LOOP|\s+WHILE)?\b", Token::Keyword),
        new_rt(r"NOT\s+NULL\b", Token::Keyword),
        new_rt(r"NULLS\s+(FIRST|LAST)\b", Token::Keyword),
        new_rt(r"UNION\s+ALL\b", Token::Keyword),
        new_rt(r"CREATE(\s+OR\s+REPLACE)?\b", Token::KeywordDDL),
        new_rt(r"DOUBLE\s+PRECISION\b", Token::NameBuiltin),
        new_rt(r"GROUP\s+BY\b", Token::Keyword),
        new_rt(r"ORDER\s+BY\b", Token::Keyword),
        new_rt(r"HANDLER\s+FOR\b", Token::Keyword),
        new_rt(r"(LATERAL\s+VIEW\s+)(EXPLODE|INLINE|PARSE_URL_TUPLE|POSEXPLODE|STACK)\b", Token::Keyword),
        new_rt(r"(AT|WITH')\s+TIME\s+ZONE\s+'[^']+'", Token::KeywordTZCast),
        new_rt(r"(NOT\s+)?(LIKE|ILIKE|RLIKE)\b", Token::OperatorComparison),
        // (r'[0-9_A-ZÀ-Ü][_$#\w]*', is_keyword),
        new_rt(r"[;:()\[\],\.]", Token::Punctuation),
        new_rt(r"[<>=~!]+", Token::OperatorComparison),
        new_rt(r"[+/@#%^&|^-]+", Token::Operator)
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_regex() {
        let regs = sql_regex();
        assert!(regs.len() > 0)
    }
}