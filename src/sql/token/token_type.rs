
#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Word,
    String,
    Reserved,
    ReservedTopLevel,
    ReservedTopLevelNoIndent,
    ReservedNewline,
    Operator,
    OpenParen,
    CloseParen,
    LineComment,
    BlockComment,
    Number,
    Placeholder,
    WhiteSpace
}