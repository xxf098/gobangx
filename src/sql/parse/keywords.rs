use std::collections::HashMap;
use regex::{Regex, RegexBuilder};
use super::TokenType;
use crate::sql::token::regex_factory::{ create_string_regex };

pub struct RegexToken {
    pub reg: Regex,
    pub typ: TokenType,
}

impl RegexToken {
    
    fn new(s: &str, typ: TokenType) -> Self {
        Self{
            reg: RegexBuilder::new(s).case_insensitive(true).build().unwrap(), 
            typ
        }
    }

    fn new_reg(r: Regex, typ: TokenType) -> Self {
        Self{
            reg: r, 
            typ
        }
    }
}

#[inline]
fn new_rt(s: &str, typ: TokenType) -> RegexToken{
    RegexToken::new(s, typ)
}


pub fn sql_regex() -> Vec<RegexToken> {
    let literal = create_string_regex(vec!["''", r#""""#, "``"]).unwrap();

    vec![
        new_rt(r"(--|# )\+.*?(\r\n|\r|\n|$)", TokenType::CommentSingleHint),
        new_rt(r"/\*\+[\s\S]*?\*/", TokenType::CommentMultilineHint),

        new_rt(r"(--|# ).*?(\r\n|\r|\n|$)", TokenType::CommentSingle),
        new_rt(r"/\*[\s\S]*?\*/", TokenType::CommentMultiline),

        new_rt(r"(\r\n|\r|\n)", TokenType::Newline),
        new_rt(r"\s+?", TokenType::Whitespace),

        new_rt(r":=", TokenType::Assignment),
        new_rt(r"::", TokenType::Punctuation),

        new_rt(r"\*", TokenType::Wildcard),

        new_rt(r"`(``|[^`])*`", TokenType::Name),
        new_rt(r"´(´´|[^´])*´", TokenType::Name),
        RegexToken::new_reg(literal, TokenType::Literal),

        new_rt(r"\?", TokenType::NamePlaceholder),
        new_rt(r"%(\(\w+\))?s", TokenType::NamePlaceholder),
        // (r'(?<!\w)[$:?]\w+', tokens.Name.Placeholder),

        new_rt(r"\\\w+", TokenType::Command),
        new_rt(r"(NOT\s+)?(IN)\b", TokenType::OperatorComparison),

        new_rt(r"(CASE|IN|VALUES|USING|FROM|AS)\b", TokenType::Keyword),

        new_rt(r"(@|##|#)[A-ZÀ-Ü]\w+", TokenType::Name),
        new_rt(r"[A-ZÀ-Ü]\w*(?:\s*\.)", TokenType::Name),
        new_rt(r"(\.)[A-ZÀ-Ü]\w*", TokenType::Name),
        new_rt(r"[A-ZÀ-Ü]\w*(?:\()", TokenType::Name),

        new_rt(r"-?0x[\dA-F]+", TokenType::NumberHexadecimal),
        new_rt(r"-?\d+(\.\d+)?E-?\d+", TokenType::NumberFloat),
        new_rt(r"-?(\d+(\.\d*)|\.\d+)", TokenType::NumberFloat),
        new_rt(r"(-\s*)?[0-9]+", TokenType::NumberInteger),

        new_rt(r"'(''|\\\\|\\'|[^'])*'", TokenType::StringSingle),
        new_rt(r#""(""|\\\\|\\"|[^"])*""#, TokenType::StringSymbol),
        new_rt(r#"(""|".*?[^\\]")"#, TokenType::StringSymbol),
        new_rt(r#"(?:[^\w\])])(\[[^\]\[]+\])"#, TokenType::Name),

        new_rt(r"((LEFT\s+|RIGHT\s+|FULL\s+)?(INNER\s+|OUTER\s+|STRAIGHT\s+)?|(CROSS\s+|NATURAL\s+)?)?JOIN\b", TokenType::Keyword),
        new_rt(r"END(\s+IF|\s+LOOP|\s+WHILE)?\b", TokenType::Keyword),
        new_rt(r"NOT\s+NULL\b", TokenType::Keyword),
        new_rt(r"NULLS\s+(FIRST|LAST)\b", TokenType::Keyword),
        new_rt(r"UNION\s+ALL\b", TokenType::Keyword),
        new_rt(r"CREATE(\s+OR\s+REPLACE)?\b", TokenType::KeywordDDL),
        new_rt(r"DOUBLE\s+PRECISION\b", TokenType::NameBuiltin),
        new_rt(r"GROUP\s+BY\b", TokenType::Keyword),
        new_rt(r"ORDER\s+BY\b", TokenType::Keyword),
        new_rt(r"HANDLER\s+FOR\b", TokenType::Keyword),
        new_rt(r"(LATERAL\s+VIEW\s+)(EXPLODE|INLINE|PARSE_URL_TUPLE|POSEXPLODE|STACK)\b", TokenType::Keyword),
        new_rt(r"(AT|WITH')\s+TIME\s+ZONE\s+'[^']+'", TokenType::KeywordTZCast),
        new_rt(r"(NOT\s+)?(LIKE|ILIKE|RLIKE)\b", TokenType::OperatorComparison),
        new_rt(r"[0-9_A-ZÀ-Ü][_$#\w]*", TokenType::KeywordRaw),
        new_rt(r"[;:()\[\],\.]", TokenType::Punctuation),
        new_rt(r"[<>=~!]+", TokenType::OperatorComparison),
        new_rt(r"[+/@#%^&|^-]+", TokenType::Operator)
    ]
}

pub fn is_keyword(k: &str) -> TokenType {
    let keyword = k.to_uppercase();
    match keyword.as_ref() {
        // KEYWORDS_COMMON
        "SELECT" | "INSERT" | "DELETE" | "UPDATE" | "UPSERT" |"REPLACE" |  "MERGE" | "DROP" | "CREATE" | "ALTER" => TokenType::KeywordDML,
        "WHERE" |"FROM" |"INNER" |"JOIN" |"STRAIGHT_JOIN" |"AND" |"OR" |"LIKE" |"ON" |"IN" |"SET" => TokenType::Keyword,
        "BY" | "GROUP" |"ORDER" |"LEFT" |"OUTER" |"FULL" => TokenType::Keyword,
        "IF" |"END" |"THEN" |"LOOP" |"AS" |"ELSE" |"FOR" |"WHILE" => TokenType::Keyword,
        "CASE" | "WHEN" | "MIN" | "MAX" | "DISTINCT" => TokenType::Keyword,
        // PostgreSQL
        "CONFLICT" | "WINDOW" | "PARTITION" | "OVER" | "PERFORM" | "NOTICE" | "PLPGSQL" | "INHERIT" | "INDEXES" | "ON_ERROR_STOP" => TokenType::Keyword,
        "BYTEA" | "BIGSERIAL" | "BIT VARYING" | "BOX"  => TokenType::Keyword,
        "CHARACTER" | "CHARACTER VARYING" | "CIDR" | "CIRCLE" => TokenType::Keyword,
        "DOUBLE PRECISION" | "INET" | "JSON" | "JSONB" | "LINE" | "LSEG" | "MACADDR" | "MONEY" => TokenType::Keyword,
        "PATH" | "PG_LSN" | "POINT" | "POLYGON" | "SMALLSERIAL" | "TSQUERY" | "TSVECTOR" | "TXID_SNAPSHOT" | "UUID" | "XML" => TokenType::Keyword,
        // KEYWORDS
        _ => TokenType::Name,
    }
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