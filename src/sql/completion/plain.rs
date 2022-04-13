use super::Completion;

pub struct Plain {
    candidates: Vec<String>,
}

impl Plain {
}


impl Completion for Plain {

    fn new(mut candidates: Vec<String>) -> Self {
        // let mut candidates: Vec<_> = candidates.iter().map(|w| w.to_string()).collect();
        candidates.sort();
        candidates.dedup();
        Self { candidates }
    }

    fn complete(&self, _full_text: String, word: &String) -> Vec<&String> {
        self.candidates.iter().filter(move |c| {
            (c.starts_with(word.to_lowercase().as_str())
                || c.starts_with(word.to_uppercase().as_str()))
                && !word.is_empty()
        }).collect::<Vec<_>>()
    }

    fn update_candidates(&mut self, candidates: &[String]) {
        for candidate in candidates {
            if self.candidates.iter().find(|x| *x == candidate).is_none() {
                self.candidates.push(candidate.clone())
            }
        }
    }
}

