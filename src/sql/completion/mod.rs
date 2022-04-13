pub mod plain;
pub mod advance;

pub trait Completion {
    fn new(candidates: Vec<String>) ->Self;
    fn complete(&self, full_text: String, word: &String) -> Vec<&String>;
    fn update_candidates(&mut self, candidates: &[String]);
}