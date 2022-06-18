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

    fn suggest_type(full_text: &str, text_before_cursor: &str) -> Vec<SuggestType>  {
        let suggest = Suggest::default();
        suggest.suggest_type(full_text, text_before_cursor)
    }

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
    fn test_select_suggests_cols_with_qualified_table_scope() {
        let suggest = Suggest::default();
        let sql = "SELECT  FROM sch.tabl";
        let types = suggest.suggest_type(sql, "SELECT ");
        assert_eq!(types.len(), 4);
        assert_eq!(types[0], SuggestType::column(Some("sch"), "tabl", None));
        assert_eq!(types[1], SuggestType::Function("".to_string()));
        assert_eq!(types[2], SuggestType::Alias(vec!["tabl".to_string()]));
        assert_eq!(types[3], SuggestType::Keyword);
    }

    #[test]
    fn test_suggest_where_suggests_columns_functions() {
        let suggest = Suggest::default();
        let sqls = vec![
            "SELECT * FROM tabl WHERE ",
            "SELECT * FROM tabl WHERE (",
            "SELECT * FROM tabl WHERE foo = ",
            "SELECT * FROM tabl WHERE bar OR ",
            "SELECT * FROM tabl WHERE foo = 1 AND ",
            "SELECT * FROM tabl WHERE (bar > 10 AND ",
            "SELECT * FROM tabl WHERE (bar AND (baz OR (qux AND (",
            "SELECT * FROM tabl WHERE 10 < ",
            "SELECT * FROM tabl WHERE foo BETWEEN ",
            "SELECT * FROM tabl WHERE foo BETWEEN foo AND "
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

    #[test]
    fn test_where_in_suggests_columns() {
        let suggest = Suggest::default();
        let sqls = vec![
            "SELECT * FROM tabl WHERE foo IN (",
            "SELECT * FROM tabl WHERE foo IN (bar, ",
            ];
        for sql in sqls {
            let now = std::time::Instant::now();
            let types = suggest.suggest_type(sql, sql);
            let elapsed = now.elapsed();
            println!("elapsed: {}ms", elapsed.as_millis());
            assert_eq!(types[0], SuggestType::column(None, "tabl", None));
            assert_eq!(types[1], SuggestType::Function("".to_string()));
            assert_eq!(types[2], SuggestType::Alias(vec!["tabl".to_string()]));
            assert_eq!(types[3], SuggestType::Keyword);
        }
    }

    #[test]
    fn test_where_equals_any_suggests_columns_or_keywords() {
        let suggest = Suggest::default();
        let sql = "SELECT * FROM tabl WHERE foo = ANY(";
        let types = suggest.suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::column(None, "tabl", None));
        assert_eq!(types[1], SuggestType::Function("".to_string()));
        assert_eq!(types[2], SuggestType::Alias(vec!["tabl".to_string()]));
        assert_eq!(types[3], SuggestType::Keyword);
    }


    #[test]
    fn test_lparen_suggests_cols() {
        let sql = "SELECT MAX( FROM tbl";
        let types = suggest_type(sql, "SELECT MAX(");
        assert_eq!(types[0], SuggestType::column(None, "tbl", None));
    }

    #[test]
    fn test_operand_inside_function_suggests_cols1() {
        let types = suggest_type("SELECT MAX(col1 +  FROM tbl", "SELECT MAX(col1 + ");
        assert_eq!(types[0], SuggestType::column(None, "tbl", None));
    }

    #[test]
    fn test_operand_inside_function_suggests_cols2() {
        let types = suggest_type("SELECT MAX(col1 + col2 +  FROM tbl", "SELECT MAX(col1 + col2 + ");
        assert_eq!(types[0], SuggestType::column(None, "tbl", None));
    }

    #[test]
    fn test_select_suggests_cols_and_funcs() {
        let types = suggest_type("SELECT ", "SELECT ");
        assert_eq!(types[0], SuggestType::Column(vec![]));
        assert_eq!(types[1], SuggestType::Function("".to_string()));
        assert_eq!(types[2], SuggestType::Alias(vec![]));
        assert_eq!(types[3], SuggestType::Keyword);
    }
    
    #[test]
    fn test_expression_suggests_tables_views_and_schemas() {
        let sqls = vec![
            "SELECT * FROM ",
            "INSERT INTO ",
            "COPY ",
            "UPDATE ",
            "DESCRIBE ",
            "DESC ",
            "EXPLAIN ",
            "SELECT * FROM foo JOIN ",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Schema(None));
            assert_eq!(types[1], SuggestType::Table("".to_string()));
            assert_eq!(types[2], SuggestType::View("".to_string()));
        }
    }

    #[test]
    fn test_expression_suggests_qualified_tables_views_and_schemas() {
        let sqls = vec![
            "SELECT * FROM sch.",
            "INSERT INTO sch.",
            "COPY sch.",
            "UPDATE sch.",
            "DESCRIBE sch.",
            "DESC sch.",
            "EXPLAIN sch.",
            "SELECT * FROM foo JOIN sch.",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Table("sch".to_string()));
            assert_eq!(types[1], SuggestType::View("sch".to_string()));
        }
      
    }
}