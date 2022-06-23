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

    fn suggest_type_multi(full_text: &str, text_before_cursor: &str) -> Vec<SuggestType>  {
        let mut suggest = Suggest::default();
        suggest._suggest_type_multi(full_text, text_before_cursor)
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

    #[test]
    fn test_truncate_suggests_tables_and_schemas() {
        let sql = "TRUNCATE ";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Schema(None));
        assert_eq!(types[1], SuggestType::Table("".to_string()));
    }

    #[test]
    fn test_truncate_suggests_qualified_tables() {
        let sql = "TRUNCATE sch.";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Table("sch".to_string()));
    }

    #[test]
    fn test_distinct_suggests_cols() {
        let sql = "SELECT DISTINCT ";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Column(vec![]));
    }

    #[test]
    fn test_col_comma_suggests_cols() {
        let sql = "SELECT a, b, FROM tbl";
        let text_before = "SELECT a, b,";
        let types = suggest_type(sql, text_before);
        assert_eq!(types[0], SuggestType::column(None, "tbl", None));
        assert_eq!(types[1], SuggestType::Function("".to_string()));
        assert_eq!(types[2], SuggestType::Alias(vec!["tbl".to_string()]));
        assert_eq!(types[3], SuggestType::Keyword);
    }

    #[test]
    fn test_table_comma_suggests_tables_and_schemas() {
        let sql = "SELECT a, b FROM tbl1, ";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Schema(None));
        assert_eq!(types[1], SuggestType::Table("".to_string()));
        assert_eq!(types[2], SuggestType::View("".to_string()));
    }

    #[test]
    fn test_into_suggests_tables_and_schemas() {
        let sql = "INSERT INTO ";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Schema(None));
        assert_eq!(types[1], SuggestType::Table("".to_string()));
        assert_eq!(types[2], SuggestType::View("".to_string()));
    }

    #[test]
    fn test_insert_into_lparen_suggests_cols() {
        let sql = "INSERT INTO abc (";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::column(None, "abc", None));
    }

    #[test]
    fn test_insert_into_lparen_partial_text_suggests_cols() {
        let sql = "INSERT INTO abc (i";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::column(None, "abc", None));
    }

    #[test]
    fn test_insert_into_lparen_comma_suggests_cols() {
        let sql = "INSERT INTO abc (id,";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::column(None, "abc", None));
    }

    #[test]
    fn test_partially_typed_col_name_suggests_col_names() {
        let sql = "SELECT * FROM tabl WHERE col_n";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::column(None, "tabl", None));
        assert_eq!(types[1], SuggestType::Function("".to_string()));
        assert_eq!(types[2], SuggestType::Alias(vec!["tabl".to_string()]));
        assert_eq!(types[3], SuggestType::Keyword);
    }

    #[test]
    fn test_dot_suggests_cols_of_a_table_or_schema_qualified_table() {
        let sql = "SELECT tabl. FROM tabl";
        let text_before = "SELECT tabl.";
        let types = suggest_type(sql, text_before);
        assert_eq!(types[0], SuggestType::column(None, "tabl", None));
        assert_eq!(types[1], SuggestType::Table("tabl".to_string()));
        assert_eq!(types[2], SuggestType::View("tabl".to_string()));
        assert_eq!(types[3], SuggestType::Function("tabl".to_string()));
    }

    #[test]
    fn test_dot_suggests_cols_of_an_alias() {
        let sql = "SELECT t1. FROM tabl1 t1, tabl2 t2";
        let text_before = "SELECT t1.";
        let types = suggest_type(sql, text_before);
        assert_eq!(types[0], SuggestType::column(None, "tabl1", Some("t1")));
        assert_eq!(types[1], SuggestType::Table("t1".to_string()));
        assert_eq!(types[2], SuggestType::View("t1".to_string()));
        assert_eq!(types[3], SuggestType::Function("t1".to_string()));
    }

    #[test]
    fn test_dot_col_comma_suggests_cols_or_schema_qualified_table() {
        let sql = "SELECT t1.a, t2. FROM tabl1 t1, tabl2 t2";
        let text_before = "SELECT t1.a, t2.";
        let types = suggest_type(sql, text_before);
        assert_eq!(types[0], SuggestType::column(None, "tabl2", Some("t2")));
        assert_eq!(types[1], SuggestType::Table("t2".to_string()));
        assert_eq!(types[2], SuggestType::View("t2".to_string()));
        assert_eq!(types[3], SuggestType::Function("t2".to_string()));
    }

    #[test]
    fn test_sub_select_suggests_keyword() {
        let sqls = vec![
            "SELECT * FROM (",
            "SELECT * FROM foo WHERE EXISTS (",
            "SELECT * FROM foo WHERE bar AND NOT EXISTS (",
            "SELECT 1 AS",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Keyword);        
        }
    }

    #[test]
    fn test_sub_select_partial_text_suggests_keyword() {
        let sqls = vec![
            "SELECT * FROM (S",
            "SELECT * FROM foo WHERE EXISTS (S",
            "SELECT * FROM foo WHERE bar AND NOT EXISTS (S",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Keyword); 
        }
    }

    #[test]
    fn test_outer_table_reference_in_exists_subquery_suggests_columns() {
        let sql = "SELECT * FROM foo f WHERE EXISTS (SELECT 1 FROM bar WHERE f.";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::column(None, "foo", Some("f")));
        assert_eq!(types[1], SuggestType::Table("f".to_string()));
        assert_eq!(types[2], SuggestType::View("f".to_string()));
        assert_eq!(types[3], SuggestType::Function("f".to_string()));
    }

    #[test]
    fn test_sub_select_table_name_completion() {
        let sqls = vec![
            "SELECT * FROM (SELECT * FROM ",
            "SELECT * FROM foo WHERE EXISTS (SELECT * FROM ",
            "SELECT * FROM foo WHERE bar AND NOT EXISTS (SELECT * FROM "
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Schema(None));
            assert_eq!(types[1], SuggestType::Table("".to_string()));
            assert_eq!(types[2], SuggestType::View("".to_string()));          
        }
    }

    #[test]
    fn test_sub_select_col_name_completion() {
        let sql = "SELECT * FROM (SELECT  FROM abc";
        let text_before = "SELECT * FROM (SELECT ";
        let types = suggest_type(sql, text_before);
        assert_eq!(types[0], SuggestType::column(None, "abc", None));
        assert_eq!(types[1], SuggestType::Function("".to_string()));
        assert_eq!(types[2], SuggestType::Alias(vec!["abc".to_string()]));
        assert_eq!(types[3], SuggestType::Keyword); 
    }

    #[test]
    fn test_sub_select_multiple_col_name_completion() {
        let sql = "SELECT * FROM (SELECT a, FROM abc";
        let text_before = "SELECT * FROM (SELECT a, ";
        let types = suggest_type(sql, text_before);
        assert_eq!(types[0], SuggestType::Column(vec![SuggestTable::new(None, "a", None), SuggestTable::new(None, "abc", None)]));
        assert_eq!(types[1], SuggestType::Function("".to_string()));
        assert_eq!(types[2], SuggestType::Alias(vec!["a".to_string(),"abc".to_string()]));
        assert_eq!(types[3], SuggestType::Keyword); 
    }

    #[test]
    fn test_sub_select_dot_col_name_completion() {
        let sql = "SELECT * FROM (SELECT t. FROM tabl t";
        let text_before = "SELECT * FROM (SELECT t.";
        let types = suggest_type(sql, text_before);
        assert_eq!(types[0], SuggestType::column(None, "tabl", Some("t")));
        assert_eq!(types[1], SuggestType::Table("t".to_string()));
        assert_eq!(types[2], SuggestType::View("t".to_string()));
        assert_eq!(types[3], SuggestType::Function("t".to_string()));
    }

    #[test]
    fn test_join_suggests_tables_and_schemas() {
        let sql = "SELECT * FROM abc INNER JOIN";
        let types = suggest_type(sql, sql);
        println!("{:?}", types);
        // FIXME
    }

    #[test]
    fn test_join_alias_dot_suggests_cols1() {
        let sqls = vec![
            "SELECT * FROM abc a JOIN def d ON a.",
            "SELECT * FROM abc a JOIN def d ON a.id = d.id AND a.",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::column(None, "abc", Some("a")));
            assert_eq!(types[1], SuggestType::Table("a".to_string()));
            assert_eq!(types[2], SuggestType::View("a".to_string()));
            assert_eq!(types[3], SuggestType::Function("a".to_string()));
        }
    }

    #[test]
    fn test_join_alias_dot_suggests_cols2() {
        let sqls = vec![
            "SELECT * FROM abc a JOIN def d ON a.id = d.",
            "SELECT * FROM abc a JOIN def d ON a.id = d.id AND a.id2 = d.",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::column(None, "def", Some("d")));
            assert_eq!(types[1], SuggestType::Table("d".to_string()));
            assert_eq!(types[2], SuggestType::View("d".to_string()));
            assert_eq!(types[3], SuggestType::Function("d".to_string()));
        }
    }

    #[test]
    fn test_on_suggests_aliases() {
        let sqls = vec![
            "select a.x, b.y from abc a join bcd b on ",
            "select a.x, b.y from abc a join bcd b on a.id = b.id OR "
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Alias(vec!["a".to_string(), "b".to_string()]));
        }
    }

    #[test]
    fn test_on_suggests_tables() {
        let sqls = vec![
            "select abc.x, bcd.y from abc join bcd on ",
            "select abc.x, bcd.y from abc join bcd on abc.id = bcd.id AND ",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Alias(vec!["abc".to_string(), "bcd".to_string()]));
        }
    }

    #[test]
    fn test_on_suggests_aliases_right_side() {
        let sqls = vec![
            "select a.x, b.y from abc a join bcd b on a.id = ",
            "select a.x, b.y from abc a join bcd b on a.id = b.id AND a.id2 = ",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Alias(vec!["a".to_string(), "b".to_string()]));
        }
    }

    #[test]
    fn test_on_suggests_tables_right_side() {
        let sqls = vec![
            "select abc.x, bcd.y from abc join bcd on ",
            "select abc.x, bcd.y from abc join bcd on abc.id = bcd.id and ",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Alias(vec!["abc".to_string(), "bcd".to_string()]));   
        }
    }

    #[test]
    fn test_join_using_suggests_common_columns() {
        let sql = "select * from abc inner join def using (";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::DropUniqueColumn(vec![SuggestTable::new(None, "abc", None), SuggestTable::new(None, "def", None)]));
    }

    #[test]
    fn test_two_join_alias_dot_suggests_cols1() {
        let sqls = vec![
            "SELECT * FROM abc a JOIN def d ON a.id = d.id JOIN ghi g ON g.",
            "SELECT * FROM abc a JOIN def d ON a.id = d.id AND a.id2 = d.id2 JOIN ghi g ON d.id = g.id AND g.",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::column(None, "ghi", Some("g")));
            assert_eq!(types[1], SuggestType::Table("g".to_string()));
            assert_eq!(types[2], SuggestType::View("g".to_string()));
            assert_eq!(types[3], SuggestType::Function("g".to_string()));
        }
    }

    #[test]
    fn test_2_statements_2nd_current() {
        let sql = "select * from a; select * from ";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Schema(None));
        assert_eq!(types[1], SuggestType::Table("".to_string()));
        assert_eq!(types[2], SuggestType::View("".to_string()));

        let sql = "select * from a; select  from b";
        let text_before = "select * from a; select ";
        let types = suggest_type_multi(sql, text_before);
        assert_eq!(types[0], SuggestType::column(None, "b", None));
        assert_eq!(types[1], SuggestType::Function("".to_string()));
        assert_eq!(types[2], SuggestType::Alias(vec!["b".to_string()]));
        assert_eq!(types[3], SuggestType::Keyword);

        let sql = "select * from; select * from ";
        let types = suggest_type_multi(sql, sql);
        assert_eq!(types[0], SuggestType::Schema(None));
        assert_eq!(types[1], SuggestType::Table("".to_string()));
        assert_eq!(types[2], SuggestType::View("".to_string()));
    }

    #[test]
    fn test_2_statements_1st_current() {
        let sql = "select * from ; select * from b";
        let text_before = "select * from ";
        let types = suggest_type_multi(sql, text_before);
        assert_eq!(types[0], SuggestType::Schema(None));
        assert_eq!(types[1], SuggestType::Table("".to_string()));
        assert_eq!(types[2], SuggestType::View("".to_string()));

        // let sql = "select  from a; select * from b";
        // let text_before = "select ";
        // let types = suggest_type_multi(sql, text_before);
        // println!("{:?}", types);
        // FIXME
    }

    #[test]
    fn test_create_db_with_template() {
        let sql = "create database foo with template ";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Database);
    }

    #[test]
    fn test_specials_included_for_initial_completion() {
        let sqls = vec!["", "    ", "\t \t"];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types[0], SuggestType::Keyword);
            assert_eq!(types[1], SuggestType::Special);
        }
    }

    #[test]
    fn test_specials_not_included_after_initial_token() {
        let sql = "create table foo (dt d";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Keyword);
    }

    #[test]
    fn test_drop_schema_qualified_table_suggests_only_tables() {
        let sql = "DROP TABLE schema_name.table_name";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Table("schema_name".to_string()));
    }

    #[test]
    fn test_cross_join() {
        let sql = "select * from v1 cross join v2 JOIN v1.id, ";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::Schema(None));
        assert_eq!(types[1], SuggestType::Table("".to_string()));
        assert_eq!(types[2], SuggestType::View("".to_string()));
    }

    #[test]
    fn test_after_as() {
        let sqls = vec![
            "SELECT 1 AS ",
            "SELECT 1 FROM tabl AS ",
            ];
        for sql in sqls {
            let types = suggest_type(sql, sql);
            assert_eq!(types.len(), 0);
        }
    }


    #[test]
    fn test_order_by() {
        let sql = "select * from foo order by ";
        let types = suggest_type(sql, sql);
        assert_eq!(types[0], SuggestType::column(None, "foo", None))
    }

}