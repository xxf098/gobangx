use regex::Regex;
use super::token_type::TokenType;
use super::regex_factory::{create_operator_regex, create_line_comment_regex, create_reserved_word_regex};

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

pub struct Token {
    typ: TokenType,
    value: String,
}

// https://regex101.com/
pub struct Tokenizer {
    whitespace_regex: Regex,
    number_regex: Regex,
    operator_regex: Regex,
    block_comment_regex: Regex,
    line_comment_regex: Regex,
    reserved_top_level_regex: Regex,
    reserved_top_level_no_indent_regex: Regex,
}

impl Tokenizer {
    pub fn new(cfg: TokenizerConfig) -> anyhow::Result<Self> {
        let t = Self {
            whitespace_regex: Regex::new(r"^(\s+)")?, 
            number_regex: Regex::new(r"^((-\s*)?[0-9]+(\.[0-9]+)?([eE]-?[0-9]+(\.[0-9]+)?)?|0x[0-9a-fA-F]+|0b[01]+)\b")?,
            operator_regex: create_operator_regex(vec!["<>", "<=", ">="])?,
            block_comment_regex: Regex::new(r"^(/\*[\S\s]*?(?:\*/|$))")?,
            line_comment_regex: create_line_comment_regex(cfg.line_comment_types)?,
            reserved_top_level_regex: create_reserved_word_regex(cfg.reserved_top_level_words)?,
            reserved_top_level_no_indent_regex: create_reserved_word_regex(cfg.reserved_top_level_words_no_indent)?,
        };
        Ok(t)
    }

    pub fn tokenize(&self) -> Vec<Token> {
        vec![]
    }
}
