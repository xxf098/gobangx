
#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Text,
    Whitespace,
    Newline,
    Error,
    Other,
    // Common token types for source code
    Keyword,
    KeywordRaw, // need check manually
    KeywordDDL,
    KeywordDML,   
    KeywordCTE,
    KeywordTZCast, 
    Name,
    NamePlaceholder,
    NameBuiltin,

    Literal,
    String,
    StringSingle,
    StringSymbol,
    Number,
    NumberHexadecimal,
    NumberFloat,
    NumberInteger,    
    Punctuation,
    Operator,
    OperatorComparison,
    Comparison,
    Wildcard,
    Comment,
    CommentSingle,
    CommentSingleHint,
    CommentMultiline,
    CommentMultilineHint,
    Assignment,
    // Generic types for non-source code
    Generic,
    Command,
    // String and some others are not direct children of Token.
    Token,
    DML,
    DDL,
    CTE,
    // group type
    Identifier,
    Where,
    Function,
    Operation,
    TypedLiteral,
    Parenthesis,
}