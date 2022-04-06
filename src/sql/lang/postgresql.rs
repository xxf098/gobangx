use crate::sql::token::tokenizer::{TokenizerConfig, Tokenize, Tokenizer};
use super::Completion;

const RESERVED_WORDS: [&str; 450] = [
  "ABORT",
  "ABSOLUTE",
  "ACCESS",
  "ACTION",
  "ADD",
  "ADMIN",
  "AFTER",
  "AGGREGATE",
  "ALL",
  "ALSO",
  "ALTER",
  "ALWAYS",
  "ANALYSE",
  "ANALYZE",
  "AND",
  "ANY",
  "ARRAY",
  "AS",
  "ASC",
  "ASSERTION",
  "ASSIGNMENT",
  "ASYMMETRIC",
  "AT",
  "ATTACH",
  "ATTRIBUTE",
  "AUTHORIZATION",
  "BACKWARD",
  "BEFORE",
  "BEGIN",
  "BETWEEN",
  "BIGINT",
  "BINARY",
  "BIT",
  "BOOLEAN",
  "BOTH",
  "BY",
  "CACHE",
  "CALL",
  "CALLED",
  "CASCADE",
  "CASCADED",
  "CASE",
  "CAST",
  "CATALOG",
  "CHAIN",
  "CHAR",
  "CHARACTER",
  "CHARACTERISTICS",
  "CHECK",
  "CHECKPOINT",
  "CLASS",
  "CLOSE",
  "CLUSTER",
  "COALESCE",
  "COLLATE",
  "COLLATION",
  "COLUMN",
  "COLUMNS",
  "COMMENT",
  "COMMENTS",
  "COMMIT",
  "COMMITTED",
  "CONCURRENTLY",
  "CONFIGURATION",
  "CONFLICT",
  "CONNECTION",
  "CONSTRAINT",
  "CONSTRAINTS",
  "CONTENT",
  "CONTINUE",
  "CONVERSION",
  "COPY",
  "COST",
  "CREATE",
  "CROSS",
  "CSV",
  "CUBE",
  "CURRENT",
  "CURRENT_CATALOG",
  "CURRENT_DATE",
  "CURRENT_ROLE",
  "CURRENT_SCHEMA",
  "CURRENT_TIME",
  "CURRENT_TIMESTAMP",
  "CURRENT_USER",
  "CURSOR",
  "CYCLE",
  "DATA",
  "DATABASE",
  "DAY",
  "DEALLOCATE",
  "DEC",
  "DECIMAL",
  "DECLARE",
  "DEFAULT",
  "DEFAULTS",
  "DEFERRABLE",
  "DEFERRED",
  "DEFINER",
  "DELETE",
  "DELIMITER",
  "DELIMITERS",
  "DEPENDS",
  "DESC",
  "DETACH",
  "DICTIONARY",
  "DISABLE",
  "DISCARD",
  "DISTINCT",
  "DO",
  "DOCUMENT",
  "DOMAIN",
  "DOUBLE",
  "DROP",
  "EACH",
  "ELSE",
  "ENABLE",
  "ENCODING",
  "ENCRYPTED",
  "END",
  "ENUM",
  "ESCAPE",
  "EVENT",
  "EXCEPT",
  "EXCLUDE",
  "EXCLUDING",
  "EXCLUSIVE",
  "EXECUTE",
  "EXISTS",
  "EXPLAIN",
  "EXPRESSION",
  "EXTENSION",
  "EXTERNAL",
  "EXTRACT",
  "FALSE",
  "FAMILY",
  "FETCH",
  "FILTER",
  "FIRST",
  "FLOAT",
  "FOLLOWING",
  "FOR",
  "FORCE",
  "FOREIGN",
  "FORWARD",
  "FREEZE",
  "FROM",
  "FULL",
  "FUNCTION",
  "FUNCTIONS",
  "GENERATED",
  "GLOBAL",
  "GRANT",
  "GRANTED",
  "GREATEST",
  "GROUP",
  "GROUPING",
  "GROUPS",
  "HANDLER",
  "HAVING",
  "HEADER",
  "HOLD",
  "HOUR",
  "IDENTITY",
  "IF",
  "ILIKE",
  "IMMEDIATE",
  "IMMUTABLE",
  "IMPLICIT",
  "IMPORT",
  "IN",
  "INCLUDE",
  "INCLUDING",
  "INCREMENT",
  "INDEX",
  "INDEXES",
  "INHERIT",
  "INHERITS",
  "INITIALLY",
  "INLINE",
  "INNER",
  "INOUT",
  "INPUT",
  "INSENSITIVE",
  "INSERT",
  "INSTEAD",
  "INT",
  "INTEGER",
  "INTERSECT",
  "INTERVAL",
  "INTO",
  "INVOKER",
  "IS",
  "ISNULL",
  "ISOLATION",
  "JOIN",
  "KEY",
  "LABEL",
  "LANGUAGE",
  "LARGE",
  "LAST",
  "LATERAL",
  "LEADING",
  "LEAKPROOF",
  "LEAST",
  "LEFT",
  "LEVEL",
  "LIKE",
  "LIMIT",
  "LISTEN",
  "LOAD",
  "LOCAL",
  "LOCALTIME",
  "LOCALTIMESTAMP",
  "LOCATION",
  "LOCK",
  "LOCKED",
  "LOGGED",
  "MAPPING",
  "MATCH",
  "MATERIALIZED",
  "MAXVALUE",
  "METHOD",
  "MINUTE",
  "MINVALUE",
  "MODE",
  "MONTH",
  "MOVE",
  "NAME",
  "NAMES",
  "NATIONAL",
  "NATURAL",
  "NCHAR",
  "NEW",
  "NEXT",
  "NFC",
  "NFD",
  "NFKC",
  "NFKD",
  "NO",
  "NONE",
  "NORMALIZE",
  "NORMALIZED",
  "NOT",
  "NOTHING",
  "NOTIFY",
  "NOTNULL",
  "NOWAIT",
  "NULL",
  "NULLIF",
  "NULLS",
  "NUMERIC",
  "OBJECT",
  "OF",
  "OFF",
  "OFFSET",
  "OIDS",
  "OLD",
  "ON",
  "ONLY",
  "OPERATOR",
  "OPTION",
  "OPTIONS",
  "OR",
  "ORDER",
  "ORDINALITY",
  "OTHERS",
  "OUT",
  "OUTER",
  "OVER",
  "OVERLAPS",
  "OVERLAY",
  "OVERRIDING",
  "OWNED",
  "OWNER",
  "PARALLEL",
  "PARSER",
  "PARTIAL",
  "PARTITION",
  "PASSING",
  "PASSWORD",
  "PLACING",
  "PLANS",
  "POLICY",
  "POSITION",
  "PRECEDING",
  "PRECISION",
  "PREPARE",
  "PREPARED",
  "PRESERVE",
  "PRIMARY",
  "PRIOR",
  "PRIVILEGES",
  "PROCEDURAL",
  "PROCEDURE",
  "PROCEDURES",
  "PROGRAM",
  "PUBLICATION",
  "QUOTE",
  "RANGE",
  "READ",
  "REAL",
  "REASSIGN",
  "RECHECK",
  "RECURSIVE",
  "REF",
  "REFERENCES",
  "REFERENCING",
  "REFRESH",
  "REINDEX",
  "RELATIVE",
  "RELEASE",
  "RENAME",
  "REPEATABLE",
  "REPLACE",
  "REPLICA",
  "RESET",
  "RESTART",
  "RESTRICT",
  "RETURNING",
  "RETURNS",
  "REVOKE",
  "RIGHT",
  "ROLE",
  "ROLLBACK",
  "ROLLUP",
  "ROUTINE",
  "ROUTINES",
  "ROW",
  "ROWS",
  "RULE",
  "SAVEPOINT",
  "SCHEMA",
  "SCHEMAS",
  "SCROLL",
  "SEARCH",
  "SECOND",
  "SECURITY",
  "SELECT",
  "SEQUENCE",
  "SEQUENCES",
  "SERIALIZABLE",
  "SERVER",
  "SESSION",
  "SESSION_USER",
  "SET",
  "SETOF",
  "SETS",
  "SHARE",
  "SHOW",
  "SIMILAR",
  "SIMPLE",
  "SKIP",
  "SMALLINT",
  "SNAPSHOT",
  "SOME",
  "SQL",
  "STABLE",
  "STANDALONE",
  "START",
  "STATEMENT",
  "STATISTICS",
  "STDIN",
  "STDOUT",
  "STORAGE",
  "STORED",
  "STRICT",
  "STRIP",
  "SUBSCRIPTION",
  "SUBSTRING",
  "SUPPORT",
  "SYMMETRIC",
  "SYSID",
  "SYSTEM",
  "TABLE",
  "TABLES",
  "TABLESAMPLE",
  "TABLESPACE",
  "TEMP",
  "TEMPLATE",
  "TEMPORARY",
  "TEXT",
  "THEN",
  "TIES",
  "TIME",
  "TIMESTAMP",
  "TO",
  "TRAILING",
  "TRANSACTION",
  "TRANSFORM",
  "TREAT",
  "TRIGGER",
  "TRIM",
  "TRUE",
  "TRUNCATE",
  "TRUSTED",
  "TYPE",
  "TYPES",
  "UESCAPE",
  "UNBOUNDED",
  "UNCOMMITTED",
  "UNENCRYPTED",
  "UNION",
  "UNIQUE",
  "UNKNOWN",
  "UNLISTEN",
  "UNLOGGED",
  "UNTIL",
  "UPDATE",
  "USER",
  "USING",
  "VACUUM",
  "VALID",
  "VALIDATE",
  "VALIDATOR",
  "VALUE",
  "VALUES",
  "VARCHAR",
  "VARIADIC",
  "VARYING",
  "VERBOSE",
  "VERSION",
  "VIEW",
  "VIEWS",
  "VOLATILE",
  "WHEN",
  "WHERE",
  "WHITESPACE",
  "WINDOW",
  "WITH",
  "WITHIN",
  "WITHOUT",
  "WORK",
  "WRAPPER",
  "WRITE",
  "XML",
  "XMLATTRIBUTES",
  "XMLCONCAT",
  "XMLELEMENT",
  "XMLEXISTS",
  "XMLFOREST",
  "XMLNAMESPACES",
  "XMLPARSE",
  "XMLPI",
  "XMLROOT",
  "XMLSERIALIZE",
  "XMLTABLE",
  "YEAR",
  "YES",
  "ZONE",
];

const RESERVED_TOP_LEVEL_WORDS: [&str; 23] = [
  "ADD",
  "AFTER",
  "ALTER COLUMN",
  "ALTER TABLE",
  "CASE",
  "DELETE FROM",
  "END",
  "EXCEPT",
  "FETCH FIRST",
  "FROM",
  "GROUP BY",
  "HAVING",
  "INSERT INTO",
  "INSERT",
  "LIMIT",
  "ORDER BY",
  "SELECT",
  "SET CURRENT SCHEMA",
  "SET SCHEMA",
  "SET",
  "UPDATE",
  "VALUES",
  "WHERE",
];

const RESERVED_TOP_LEVEL_WORDS_NO_INDENT: [&str; 4] = ["INTERSECT", "INTERSECT ALL", "UNION", "UNION ALL"];

const RESERVED_NEW_LINE_WORDS: [&str; 14] = [
  "AND",
  "ELSE",
  "OR",
  "WHEN",
  // joins
  "JOIN",
  "INNER JOIN",
  "LEFT JOIN",
  "LEFT OUTER JOIN",
  "RIGHT JOIN",
  "RIGHT OUTER JOIN",
  "FULL JOIN",
  "FULL OUTER JOIN",
  "CROSS JOIN",
  "NATURAL JOIN",
];  


pub struct PostgreSQL {

}


impl Tokenize for PostgreSQL {
    fn tokenizer(&self) -> anyhow::Result<Tokenizer> {
        let cfg = TokenizerConfig{
            reserved_words: RESERVED_WORDS.to_vec(),
            reserved_top_level_words: RESERVED_TOP_LEVEL_WORDS.to_vec(),
            reserved_newline_words: RESERVED_NEW_LINE_WORDS.to_vec(),
            reserved_top_level_words_no_indent: RESERVED_TOP_LEVEL_WORDS_NO_INDENT.to_vec(),
            // string_types: vec![r#""""#, "''", r"U&''", r#"U&"""#, "$$"], // FIXME
            string_types: vec![r#""""#, "''", r"U&''", r#"U&"""#,],
            open_parens: vec!["(", "CASE"],
            close_parens: vec![")", "END"],
            indexed_placeholder_types: vec!["$"],
            named_placeholder_types: vec![":"],
            line_comment_types: vec!["--"],
            special_word_chars: vec![],
            operator: vec![
                "!=",
                "<<",
                ">>",
                "||/",
                "|/",
                "::",
                "->>",
                "->",
                "~~*",
                "~~",
                "!~~*",
                "!~~",
                "~*",
                "!~*",
                "!~",
                "!!",
            ],
        };
        Tokenizer::new(cfg)
    }
}


impl Completion for PostgreSQL {
    fn complete() -> Vec<&'static str> {
        RESERVED_WORDS.to_vec().into_iter()
        .chain(RESERVED_TOP_LEVEL_WORDS.to_vec().into_iter())
        .chain(RESERVED_NEW_LINE_WORDS.to_vec().into_iter())
        .chain(RESERVED_TOP_LEVEL_WORDS_NO_INDENT.to_vec().into_iter())
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgre_sql() {
        let s = PostgreSQL{};
        let t = s.tokenizer().unwrap();
        let sql = "select * from users limit 10;";
        let tokens = t.tokenize(sql);
        for token in tokens {
            println!("{:?}", token);
        }
    }
}