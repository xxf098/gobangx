use regex::Regex;
use super::token_type::TokenType;
use super::regex_factory;

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
    reserved_newline_regex: Regex,
    reserved_plain_regex: Regex,
    word_regex: Regex,
    string_regex: Regex,
    open_paren_regex: Regex,
    close_paren_regex: Regex,
    indexed_placeholder_regex: Option<Regex>,
    ident_named_placeholder_regex: Option<Regex>,
    string_named_placeholder_regex: Option<Regex>,
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
            indexed_placeholder_regex: regex_factory::create_placeholder_regex(cfg.indexed_placeholder_types, r"[0-9]*").ok(),
            ident_named_placeholder_regex: regex_factory::create_placeholder_regex(cfg.named_placeholder_types.clone(), r"[a-zA-Z0-9._$]+").ok(),
            string_named_placeholder_regex: regex_factory::create_placeholder_regex(cfg.named_placeholder_types, &regex_factory::create_string_pattern(cfg.string_types)).ok(),
        };
        Ok(t)
    }

    pub fn tokenize(&self) -> Vec<Token> {
        vec![]
    }
}
