use super::Filter;
use crate::lexer::Token;
use crate::tokens::TokenType;

pub enum Case {
    Upper,
    Lower,
    Origin, // keep origin
}

impl From<&str> for Case {
    fn from(case: &str) -> Self {
        match case {
            "upper" => Case::Upper,
            "lower" => Case::Lower,
            _ => Case::Origin,
        }
    }
}

pub struct KeywordCaseFilter {
    case: Case,
}

impl KeywordCaseFilter {

    pub fn new(case: &str) -> Self {
        Self { case: case.into() }
    }
}

impl Filter for KeywordCaseFilter {

    fn process(&self, token: &mut Token, _depth: usize) {
        if token.is_keyword() {
             match self.case {
                Case::Upper => { token.value = token.value.to_uppercase() },
                Case::Lower => { token.value = token.value.to_lowercase() },
                _ => {},
            };
        }
    }
}


pub struct IdentifierCaseFilter{
    case: Case,
}

impl IdentifierCaseFilter {

    pub fn new(case: &str) -> Self {
        Self { case: case.into() }
    }
}

impl Filter for IdentifierCaseFilter {

    fn process(&self, token: &mut Token, _depth: usize) { 
        if token.typ == TokenType::Name || token.typ == TokenType::StringSymbol {
            if !token.value.starts_with("\"") {
                match self.case {
                    Case::Upper => { token.value = token.value.to_uppercase() },
                    Case::Lower => { token.value = token.value.to_lowercase() },
                    _ => {},
                };
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_case_filter() {
        let f = KeywordCaseFilter::new("upper");
        let mut t = Token::new(TokenType::Keyword, "select");
        f.process(&mut t, 0);
        assert_eq!(t.value, "SELECT");
    }
}