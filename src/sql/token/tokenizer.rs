use std::string::ToString;
use regex::Regex;
use super::token_type::TokenType;
use super::regex_factory;

macro_rules! check_some {
    ($opt: expr) => {
        if let Some(v) = $opt {
            return Some(v)
        } 
    };
}


pub struct TokenizerConfig<'a> {
    pub reserved_words: Vec<&'a str>,
    pub reserved_top_level_words: Vec<&'a str>,
    pub reserved_newline_words: Vec<&'a str>,
    pub reserved_top_level_words_no_indent: Vec<&'a str>,
    pub string_types: Vec<&'a str>,
    pub open_parens: Vec<&'a str>,
    pub close_parens: Vec<&'a str>,
    pub indexed_placeholder_types: Vec<&'a str>,
    pub named_placeholder_types: Vec<&'a str>,
    pub line_comment_types: Vec<&'a str>,
    pub special_word_chars: Vec<&'a str>,
    pub operator: Vec<&'a str>,
}



pub trait Tokenize {
    fn tokenizer(&self) -> anyhow::Result<Tokenizer>;
}

#[derive(Clone, Debug)]
pub struct Token {
    pub typ: TokenType,
    pub key: Option<String>,
    pub value: String,
    pub whitespace_before: String,
}

impl Token {

    pub fn new(typ: TokenType, value: &str) -> Self {
        Self { 
            typ,
            value: value.to_string(),
            key: None,
            whitespace_before: "".to_string(),
         }
    }

    pub fn set_whitespace(&mut self, n: usize) {
        if n > 0 {
            self.whitespace_before = " ".repeat(n)
        }
    }
}

impl ToString for Token {

    fn to_string(&self) -> String {
        format!("{}{}", self.whitespace_before, self.value)
    }
}

pub struct Tokenizer {
    whitespace_regex: Regex,
    number_regex: Regex,
    operator_regex: Regex,
    block_comment_regex: Regex,
    line_comment_regex: Regex,
    reserved_top_level_regex: Regex,
    reserved_top_level_no_indent_regex: Regex,
    reserved_newline_regex: Regex,
    reserved_plain_regex: Regex,
    word_regex: Regex,
    string_regex: Regex,
    open_paren_regex: Regex,
    close_paren_regex: Regex,
    // indexed_placeholder_regex: Option<Regex>,
    // ident_named_placeholder_regex: Option<Regex>,
    // string_named_placeholder_regex: Option<Regex>,
}

impl Tokenizer {
    pub fn new(cfg: TokenizerConfig) -> anyhow::Result<Self> {
        let t = Self {
            whitespace_regex: Regex::new(r"^(\s+)")?, 
            number_regex: Regex::new(r"^((-\s*)?[0-9]+(\.[0-9]+)?([eE]-?[0-9]+(\.[0-9]+)?)?|0x[0-9a-fA-F]+|0b[01]+)\b")?,
            operator_regex: regex_factory::create_operator_regex(vec!["<>", "<=", ">="])?,
            block_comment_regex: Regex::new(r"^(/\*[\S\s]*?(?:\*/|$))")?,
            line_comment_regex: regex_factory::create_line_comment_regex(cfg.line_comment_types)?,
            reserved_top_level_regex: regex_factory::create_reserved_word_regex(cfg.reserved_top_level_words)?,
            reserved_top_level_no_indent_regex: regex_factory::create_reserved_word_regex(cfg.reserved_top_level_words_no_indent)?,
            reserved_newline_regex: regex_factory::create_reserved_word_regex(cfg.reserved_newline_words)?,
            reserved_plain_regex: regex_factory::create_reserved_word_regex(cfg.reserved_words)?,
            word_regex: regex_factory::create_word_regex(cfg.special_word_chars)?,
            string_regex: regex_factory::create_string_regex(cfg.string_types.clone())?,
            open_paren_regex: regex_factory::create_paren_regex(cfg.open_parens)?,
            close_paren_regex: regex_factory::create_paren_regex(cfg.close_parens)?,
            // indexed_placeholder_regex: regex_factory::create_placeholder_regex(cfg.indexed_placeholder_types, r"[0-9]*").ok(),
            // ident_named_placeholder_regex: regex_factory::create_placeholder_regex(cfg.named_placeholder_types.clone(), r"[a-zA-Z0-9._$]+").ok(),
            // string_named_placeholder_regex: regex_factory::create_placeholder_regex(cfg.named_placeholder_types, &regex_factory::create_string_pattern(cfg.string_types)).ok(),
        };
        Ok(t)
    }

    pub fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut token: Option<Token> = None;
        let mut index = 0;
        let input_len = input.len();
        while index < input_len {
            let whitespace_before = self.get_whitespace_count(&input[index..]);
            index += whitespace_before;
            if index < input_len {
                token = self.get_next_token(&input[index..], token);
                if let Some(t) = token.as_mut() {
                    index += t.value.len();
                    t.set_whitespace(whitespace_before);
                    tokens.push(t.clone());
                }
            }
        }
        tokens
    }

    fn get_whitespace_count(&self, input: &str) -> usize {
        self.whitespace_regex.find(input).map(|s| s.as_str().len()).unwrap_or(0)
    }

    fn get_next_token(&self, input: &str, previous_token: Option<Token>) -> Option<Token> {
        check_some!(self.get_comment_token(input));
        check_some!(self.get_string_token(input));
        check_some!(self.get_open_paren_token(input));
        check_some!(self.get_close_paren_token(input));
        check_some!(self.get_placeholder_token(input));
        check_some!(self.get_number_token(input));
        check_some!(self.get_reserved_word_token(input, previous_token));
        check_some!(self.get_word_token(input));
        self.get_operator_token(input)
    }

    fn get_comment_token(&self, input: &str) -> Option<Token> {
        check_some!(self.get_line_comment_token(input));
        self.get_block_comment_token(input)
    }

    fn get_line_comment_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.line_comment_regex, TokenType::LineComment)
    }

    fn get_block_comment_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.block_comment_regex, TokenType::BlockComment)
    }

    fn get_string_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.string_regex, TokenType::String)
    }

    fn get_open_paren_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.open_paren_regex, TokenType::OpenParen)
    }

    fn get_close_paren_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.close_paren_regex, TokenType::CloseParen)
    }

    // TODO:
    fn get_placeholder_token(&self, _input: &str) -> Option<Token> {
        None
    }

    fn get_number_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.number_regex, TokenType::Number)
    }

    fn get_word_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.word_regex, TokenType::Word)
    }

    fn get_operator_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.operator_regex, TokenType::Operator)
    }

    fn get_reserved_word_token(&self, input: &str, previous_token: Option<Token>) -> Option<Token> {
        if let Some(prev) = previous_token {
            if prev.value == "." {  return None }
        };
        check_some!(self.get_top_level_reserved_token(input));
        check_some!(self.get_newline_reserved_token(input));
        check_some!(self.get_top_level_reserved_token_no_indent(input));
        self.get_plain_reserved_token(input)
    }

    fn get_top_level_reserved_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.reserved_top_level_regex, TokenType::ReservedTopLevel)
    }

    fn get_newline_reserved_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.reserved_newline_regex, TokenType::ReservedNewline)
    }

    fn get_top_level_reserved_token_no_indent(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.reserved_top_level_no_indent_regex, TokenType::ReservedTopLevelNoIndent)
    }

    fn get_plain_reserved_token(&self, input: &str) -> Option<Token> {
        get_token_on_first_match(input, &self.reserved_plain_regex, TokenType::Reserved)
    }

}

fn get_token_on_first_match(input: &str, reg: &Regex, typ: TokenType) -> Option<Token> {
    reg.find(input).map(|m| Token::new(typ, m.as_str()))
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::lang::Standard;

    #[test]
    fn test_get_line_comment_token() {
        let standard = Standard{};
        let t = standard.tokenizer().unwrap();
        let input = "-- select * from users;";
        let token = t.get_line_comment_token(input).unwrap();
        assert_eq!(token.typ, TokenType::LineComment);
        assert_eq!(token.value, input);
    }

    #[test]
    fn test_get_block_comment_token() {
        let standard = Standard{};
        let t = standard.tokenizer().unwrap();
        let input = r#"/*Select all the columns
        of all the records
        in the Customers table:*/"#;
        let token = t.get_block_comment_token(input).unwrap();
        assert_eq!(token.typ, TokenType::BlockComment);
        assert_eq!(token.value, input);
    }

    #[test]
    fn test_get_string_token() {
        let standard = Standard{};
        let t = standard.tokenizer().unwrap();
        let input = r"'value' ";
        let token = t.get_string_token(input).unwrap();
        assert_eq!(token.typ, TokenType::String);
        assert_eq!(token.value, r#"'value'"#)
    }

    #[test]
    fn test_get_open_paren_token() {
        let standard = Standard{};
        let t = standard.tokenizer().unwrap();
        let input = r"(abc) ";
        let token = t.get_open_paren_token(input).unwrap();
        assert_eq!(token.typ, TokenType::OpenParen);
        assert_eq!(token.value, r"(")
    }

    #[test]
    fn test_get_close_paren_token() {
        let standard = Standard{};
        let t = standard.tokenizer().unwrap();
        let input = r")  ";
        let token = t.get_close_paren_token(input).unwrap();
        assert_eq!(token.typ, TokenType::CloseParen);
        assert_eq!(token.value, r")")
    }

    #[test]
    fn test_get_number_token() {
        let standard = Standard{};
        let t = standard.tokenizer().unwrap();
        let input = r"987654";
        let token = t.get_number_token(input).unwrap();
        assert_eq!(token.typ, TokenType::Number);
        assert_eq!(token.value, input)
    }

    #[test]
    fn test_get_word_token() {
        let standard = Standard{};
        let t = standard.tokenizer().unwrap();
        let input = r"word";
        let token = t.get_word_token(input).unwrap();
        assert_eq!(token.typ, TokenType::Word);
        assert_eq!(token.value, input)
    }

    #[test]
    fn test_get_operator_token() {
        let standard = Standard{};
        let t = standard.tokenizer().unwrap();
        let input = r">=";
        let token = t.get_operator_token(input).unwrap();
        assert_eq!(token.typ, TokenType::Operator);
        assert_eq!(token.value, input)
    }
}