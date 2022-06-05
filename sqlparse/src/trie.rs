use std::collections::HashMap;
use super::TokenType;

struct TrieNode {
    typ: Option<TokenType>,
    is_last: bool,
    children: HashMap<char, TrieNode>,
}

impl TrieNode {

    fn new() -> Self {
        Self {
            typ: None,
            is_last: false,
            children: HashMap::new(),
        }
    }
}


pub struct Trie {
    root: TrieNode,
}

impl Default for Trie {
    fn default() -> Self {
        Self { root: TrieNode::new() }
    }
}


impl Trie {

    pub fn _insert(&mut self, key: &str) {
        let chars = key.chars();
        let mut current = &mut self.root;
        for c in chars {
            if !current.children.contains_key(&c) {
                current.children.insert(c, TrieNode::new());
            }
            current = current.children.get_mut(&c).unwrap();
        }
        current.is_last = true;
    }

    pub fn insert_token(&mut self, key: &str, typ: TokenType) {
        let chars = key.chars();
        let mut current = &mut self.root;
        for c in chars {
            if !current.children.contains_key(&c) {
                current.children.insert(c, TrieNode::new());
            }
            current = current.children.get_mut(&c).unwrap();
        }
        current.typ = Some(typ);
        current.is_last = true;
    }


    pub fn _search(&self, key: &str) -> bool {
        let chars = key.chars();
        let mut current = &self.root;
        for c in chars {
            if !current.children.contains_key(&c) {
                return false
            }
            current = current.children.get(&c).unwrap();
        }
        current.is_last
    }

    // match a-z 0-9
    pub fn _match_keyword(&self, sql: &str) -> Option<usize> {
        let chars = sql.chars();
        let mut current = &self.root;
        for (level, c) in chars.enumerate() {
            if !current.children.contains_key(&c) {
                if level < 3 { return None; } // min keyword length is 2
                // https://www.regular-expressions.info/wordboundaries.html
                let is_end = match c {
                    ' ' | ';' | ':' | '\n' | '\r' | '(' | ')' => true,
                    _ => false,
                };
                return if is_end { Some(level) } else { None }
            }
            current = current.children.get(&c).unwrap();
        }
        if current.is_last { Some(sql.len()) } else { None }
    }

    pub fn match_token(&self, sql: &str) -> Option<(usize, Option<TokenType>)> {
        let chars = sql.chars();
        let mut current = &self.root;
        for (level, c) in chars.enumerate() {
            if !current.children.contains_key(&c) {
                if level < 3 { return match_name(sql) } // min keyword length is 2
                // https://www.regular-expressions.info/wordboundaries.html
                let is_end = match c {
                    ' ' | ';' | ':' | '\n' | '\r' | '(' | ')' => true,
                    _ => false,
                };
                return if is_end { Some((level, current.typ.clone())) } else { match_name(sql) }
            }
            current = current.children.get(&c).unwrap();
        }
        if current.is_last { Some((sql.len(), current.typ.clone())) } else { match_name(sql) }
    }

}

fn match_name(sql: &str) -> Option<(usize, Option<TokenType>)> {
    // match 0-9A-Z_ AND end with space ;
    let chars = sql.chars();
    let mut last_level = 0;
    let mut last_char = ';';
    for (level, c) in chars.enumerate() {
       let is_word_character = (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_';
       last_level = level;
       last_char = c;
       if !is_word_character { break; }
    }
    // println!("t: {}, last_level {}, last_char {}", sql, last_level, last_char);
    if last_level > 0 && (last_char == ' ' || last_char == ';' || last_char == ',' || last_char == ')' || last_char == '\n') { 
        Some((last_level, Some(TokenType::Name))) } else { None }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char() {
        let c = '0';
        assert!(c >= '0');
        let c = '2';
        assert!(c >= '0' && c <= '9');
        let c = ';';
        assert_eq!(c >= '0' && c <= '9', false);
    }

    #[test]
    fn test_trie() {
        let mut t = Trie::default();
        t._insert("the");
        t._insert("a");
        t._insert("there");
        t._insert("answer");
        t._insert("any");
        t._insert("bye");
        t._insert("by");
        t._insert("their");
        assert!(t._search("the"));
        assert!(!t._search("these"));
        assert!(t._search("there"));
        assert!(!t._search("thaw"));
    }

    #[test]
    fn test_match_keyword() {
        let mut t = Trie::default();
        t._insert("SELECT");
        t._insert("WHERE");
        t._insert("FROM");
        t._insert("ON");
        t._insert("IN");
        t._insert("CASE");
        t._insert("WHEN");
        let sql = "SELECT * FROM foo.bar";
        let pos = t._match_keyword(sql).unwrap();
        assert_eq!(&sql[0..pos], "SELECT");
    }

    #[test]
    fn test_match_token() {
        let mut t = Trie::default();
        t.insert_token("SELECT", TokenType::KeywordDML);
        t.insert_token("WHERE", TokenType::Keyword);
        t.insert_token("FROM", TokenType::Keyword);
        t.insert_token("ON", TokenType::Keyword);
        t.insert_token("IN", TokenType::Keyword);
        t.insert_token("CASE", TokenType::Keyword);
        t.insert_token("WHEN", TokenType::Keyword);
        let sql = "SELECT * FROM foo.bar";
        let (pos, typ) = t.match_token(sql).unwrap();
        assert_eq!(&sql[0..pos], "SELECT");
        assert_eq!(typ.unwrap(), TokenType::KeywordDML);
        let sql = "FROM foo.bar";
        let (pos, typ) = t.match_token(sql).unwrap();
        assert_eq!(&sql[0..pos], "FROM");
        assert_eq!(typ.unwrap(), TokenType::Keyword);
    }

    #[test]
    fn test_trie1() {
        let mut t = Trie::default();
        t._insert("apple");
        assert!(t._search("apple"));
        assert!(!t._search("app"));
        t._insert("app");
        assert!(t._search("app"));
    }


}