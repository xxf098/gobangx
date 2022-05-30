use super::engine::FilterStack;
use super::filters::{
    Filter, StmtFilter, TokenListFilter,
    KeywordCaseFilter, IdentifierCaseFilter, StripWhitespaceFilter, ReindentFilter, AlignedIndentFilter, StripBeforeNewline,
};

#[derive(Default)]
pub struct FormatOption<'a> {
    pub keyword_case: &'a str,
    pub identifier_case: &'a str,
    pub output_format: &'a str,
    pub strip_comments: bool,
    pub use_space_around_operators: bool,
    pub strip_whitespace: bool,
    // reindent
    pub reindent: bool,
    pub indent_columns: bool,
    pub reindent_aligned: bool,
    pub indent_after_first: bool,
    pub indent_tabs: bool,
    pub indent_width: usize,
    pub indent_char: &'a str,
    pub wrap_after: usize,
    pub comma_first: bool,
    pub right_margin: bool,
    pub grouping: bool,
}

impl<'a> FormatOption<'a> {

    pub fn default_reindent() -> Self {
        let mut options = Self::default();
        options.reindent = true;
        options.indent_width = 2;
        options.indent_char = " ";
        options
    }

    pub fn default_reindent_aligned() -> Self {
        let mut options = Self::default();
        options.reindent_aligned = true;
        // options.indent_char = " ";
        options
    }
}

pub fn validate_options(options: &mut FormatOption) {
    if options.reindent_aligned {
        options.strip_whitespace = true
    }
    options.indent_char = " ";
}


pub fn build_filter_stack(stack: &mut FilterStack, options: &mut FormatOption) {
    if options.keyword_case.len() > 0 {
        let filter = Box::new(KeywordCaseFilter::new("upper")) as Box<dyn Filter>;
        stack.preprocess.push(filter);
    }
    if options.identifier_case.len() > 0 {
        let filter = Box::new(IdentifierCaseFilter::new("upper")) as Box<dyn Filter>;
        stack.preprocess.push(filter);
    }
    if options.strip_whitespace {
        options.grouping = true;
        let filter = Box::new(StripWhitespaceFilter{}) as Box<dyn StmtFilter>;
        stack.stmtprocess.push(filter);
    }
    if options.reindent {
        options.grouping = true;
        // width, char, wrap_after, n, comma_first, indent_after_first, indent_columns
        let filter = ReindentFilter::new(
            options.indent_width,
            options.indent_char,
            options.wrap_after,
            "\n", 
            options.comma_first,
            options.indent_after_first, 
            options.indent_columns);
        let filter = Box::new(filter) as Box<dyn TokenListFilter>;
        stack.tlistprocess.push(filter);
    }

    if options.reindent_aligned {
        options.grouping = true;
        let filter = AlignedIndentFilter::new(options.indent_char, "\n");
        let filter = Box::new(filter) as Box<dyn TokenListFilter>;
        stack.tlistprocess.push(filter);
    }

    let filter = Box::new(StripBeforeNewline{}) as Box<dyn StmtFilter>;
    stack.postprocess.push(filter);
}