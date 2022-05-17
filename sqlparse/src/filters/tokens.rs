use super::Filter;
use crate::lexer::Token;

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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::TokenType;

    #[test]
    fn test_keyword_case_filter() {
        let f = KeywordCaseFilter::new("upper");
        let mut t = Token::new(TokenType::Keyword, "select");
        f.process(&mut t);
        assert_eq!(t.value, "SELECT");
    }
}