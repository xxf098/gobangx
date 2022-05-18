use super::engine::FilterStack;
use super::filters::{Filter};
use super::filters::tokens::{KeywordCaseFilter, IdentifierCaseFilter};

#[derive(Default)]
pub struct FormatOption<'a> {
    pub keyword_case: &'a str,
    pub identifier_case: &'a str,
    pub output_format: &'a str,
    pub strip_comments: bool,
    pub use_space_around_operators: bool,
    pub strip_whitespace: bool,
    pub indent_columns: bool,
    pub reindent: bool,
    pub reindent_aligned: bool,
    pub indent_after_first: bool,
    pub indent_tabs: bool,
    pub indent_width: bool,
    pub wrap_after: bool,
    pub comma_first: bool,
    pub right_margin: bool,
}

pub fn validate_options(options: &mut FormatOption) {
}


pub fn build_filter_stack(mut stack: FilterStack, options: &FormatOption) -> FilterStack {
    if options.keyword_case.len() > 0 {
        let filter = Box::new(KeywordCaseFilter::new("upper")) as Box<dyn Filter>;
        stack.preprocess.push(filter);
    }
    if options.identifier_case.len() > 0 {
        let filter = Box::new(IdentifierCaseFilter::new("upper")) as Box<dyn Filter>;
        stack.preprocess.push(filter);
    }
    stack
}