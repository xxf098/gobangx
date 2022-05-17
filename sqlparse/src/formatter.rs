use super::engine::FilterStack;

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

pub fn validate_options(options: FormatOption) -> FormatOption {
    options
}


pub fn build_filter_stack(stack: FilterStack, options: &FormatOption) -> FilterStack {
    
    stack
}