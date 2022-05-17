use super::Filter;
use crate::lexer::Token;
use crate::tokens::TokenType;

pub enum Case {
    Upper,
    Lower,
}

pub struct KeywordCaseFilter {
    case: Case,
}

impl KeywordCaseFilter {

    pub fn new(case: &str) -> Self {
        Self { case: if case == "upper" { Case::Upper } else { Case::Lower } }
    }
}

impl Filter for KeywordCaseFilter {

    fn process(&self, token: &mut Token) {
        if token.is_keyword() {
            token.value = match self.case {
                Case::Upper => { token.value.to_uppercase() },
                Case::Lower => { token.value.to_lowercase() }
            };
        }
    }
}


pub struct IdentifierCaseFilter{
    case: Case,
}

impl IdentifierCaseFilter {

    pub fn new(case: &str) -> Self {
        Self { case: if case == "upper" { Case::Upper } else { Case::Lower } }
    }
}

impl Filter for IdentifierCaseFilter {

    fn process(&self, token: &mut Token) { 
        if token.typ == TokenType::Name || token.typ == TokenType::StringSymbol {
            if !token.value.starts_with("\"") {
                token.value = match self.case {
                    Case::Upper => { token.value.to_uppercase() },
                    Case::Lower => { token.value.to_lowercase() }
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
        f.process(&mut t);
        assert_eq!(t.value, "SELECT");
    }
}