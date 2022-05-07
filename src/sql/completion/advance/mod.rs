pub mod completion_engine;
pub mod sqlcompleter;
pub mod utils;
pub mod suggest;
pub mod updater;

pub use utils::{last_word, find_prev_keyword, extract_tables};
pub use updater::Updater;
use suggest::{SuggestTable, SuggestType, suggest_type};