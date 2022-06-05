use std::collections::HashMap;

struct TrieNode {
    value: Option<char>,
    is_last: bool,
    children: HashMap<char, TrieNode>,
}

impl TrieNode {

    fn new(value: Option<char>) -> Self {
        Self {
            value,
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
        Self { root: TrieNode::new(None) }
    }
}


impl Trie {

    pub fn insert(&mut self, key: &str) {
        let chars = key.chars();
        let mut current = &mut self.root;
        for c in chars {
            if !current.children.contains_key(&c) {
                current.children.insert(c, TrieNode::new(Some(c)));
            }
            current = current.children.get_mut(&c).unwrap();
        }
        current.is_last = true;
    }


    pub fn search(&self, key: &str) -> bool {
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

    // return keyword type 
    // match a-z 0-9
    pub fn match_keyword(&self, sql: &str) -> Option<usize> {
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

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie() {
        let mut t = Trie::default();
        t.insert("the");
        t.insert("a");
        t.insert("there");
        t.insert("answer");
        t.insert("any");
        t.insert("bye");
        t.insert("by");
        t.insert("their");
        assert!(t.search("the"));
        assert!(!t.search("these"));
        assert!(t.search("there"));
        assert!(!t.search("thaw"));
    }

    #[test]
    fn test_match_keyword() {
        let mut t = Trie::default();
        t.insert("SELECT");
        t.insert("WHERE");
        t.insert("FROM");
        t.insert("ON");
        t.insert("IN");
        t.insert("CASE");
        t.insert("WHEN");
        let sql = "SELECT * FROM foo.bar";
        let pos = t.match_keyword(sql).unwrap();
        assert_eq!(&sql[0..pos], "SELECT");
    }

    #[test]
    fn test_trie1() {
        let mut t = Trie::default();
        t.insert("apple");
        assert!(t.search("apple"));
        assert!(!t.search("app"));
        t.insert("app");
        assert!(t.search("app"));
    }


}