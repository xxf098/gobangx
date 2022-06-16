pub mod sqlcompleter;
pub mod utils;
pub mod completion_engine;
pub mod updater;

pub use utils::{last_word, find_prev_keyword, extract_tables};
pub use updater::Updater;
use completion_engine::{SuggestTable, SuggestType, Suggest};


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_suggests_cols_with_visible_table_scope() {
        let suggest = Suggest::default();
        let sql = "SELECT  FROM tabl";
        let types = suggest.suggest_type(sql, "SELECT ");
        assert_eq!(types.len(), 4);
        assert_eq!(types[0], SuggestType::column(None, "tabl", None));
        assert_eq!(types[1], SuggestType::Function("".to_string()));
        assert_eq!(types[2], SuggestType::Alias(vec!["tabl".to_string()]));
        assert_eq!(types[3], SuggestType::Keyword);
    }

    #[test]
    fn test_suggest_where_suggests_columns_functions() {
        let suggest = Suggest::default();
        let sqls = vec![
            "SELECT * FROM tabl WHERE ",
            // "SELECT * FROM tabl WHERE ("
        ];
        for sql in sqls {
            let types = suggest.suggest_type(sql, sql);
            // println!("{:?}", types);
            assert_eq!(types[0], SuggestType::column(None, "tabl", None));
            assert_eq!(types[1], SuggestType::Function("".to_string()));
            assert_eq!(types[2], SuggestType::Alias(vec!["tabl".to_string()]));
            assert_eq!(types[3], SuggestType::Keyword);
        }
    }
}