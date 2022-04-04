use regex::Regex;
use super::token_type::TokenType;

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
}

pub trait Tokenize {
    fn tokenizer(&self) -> anyhow::Result<Tokenizer>;
}

pub struct Token {
    typ: TokenType,
    value: String,
}

pub struct Tokenizer {
    whitespace_regex: Regex,
    number_regex: Regex,
}

impl Tokenizer {
    pub fn new(cfg: TokenizerConfig) -> anyhow::Result<Self> {
        let t = Self {
            whitespace_regex: Regex::new(r"^(\s+)")?, 
            number_regex: Regex::new(r"^((-\s*)?[0-9]+(\.[0-9]+)?([eE]-?[0-9]+(\.[0-9]+)?)?|0x[0-9a-fA-F]+|0b[01]+)\b")?,

        };
        Ok(t)
    }

    pub fn tokenize(&self) -> Vec<Token> {
        vec![]
    }
}
