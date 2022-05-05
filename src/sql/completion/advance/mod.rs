pub mod completion_engine;
pub mod sqlcompleter;
pub mod utils;
pub mod suggest;

pub use utils::{last_word, find_prev_keyword, extract_tables};
use suggest::SuggestTable;