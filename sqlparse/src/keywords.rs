// use std::collections::HashMap;
use regex::{Regex};
use super::{TokenType, trie::Trie};
// use crate::sql::token::regex_factory::{ create_string_regex };

pub struct RegexToken {
    pub reg: Regex,
    pub typ: TokenType,
    pub capture: Option<usize>,
    pub backward: usize, // backward offset, match .Name
    pub needle: Option<&'static str>,
    pub shortest: bool,
}

impl RegexToken {
    
    fn new(s: &str, typ: TokenType, capture: Option<usize>, backward: usize, shortest: bool) -> Self {
        // let reg = if ignore_case { RegexBuilder::new(s).case_insensitive(true).build().unwrap() } else { Regex::new(s).unwrap() };
        let reg = Regex::new(s).unwrap();
        Self{
            reg, 
            typ,
            capture,
            backward,
            needle: None,
            shortest: shortest,
        }
    }

    fn new_nd(s: &str, typ: TokenType, capture: Option<usize>, backward: usize, needle: &'static str, shortest: bool) -> Self {
        // let reg = if ignore_case { RegexBuilder::new(s).case_insensitive(true).build().unwrap() } else { Regex::new(s).unwrap() };
        let reg = Regex::new(s).unwrap();
        Self{
            reg, 
            typ,
            capture,
            backward,
            needle: Some(needle),
            shortest: shortest,
        }
    }

    fn _new_reg(r: Regex, typ: TokenType) -> Self {
        Self{
            reg: r, 
            typ,
            capture: None,
            backward: 0,
            needle: None,
            shortest: false,
        }
    }
}

#[inline]
fn new_rt(s: &str, typ: TokenType) -> RegexToken{
    RegexToken::new(s, typ, None, 0, false)
}

#[inline]
fn new_nd(s: &str, typ: TokenType, probe: &'static str) -> RegexToken{
    RegexToken::new_nd(s, typ, None, 0, probe, false)
}

#[inline]
fn new_sh(s: &str, typ: TokenType, needle: &'static str) -> RegexToken{
    RegexToken::new_nd(s, typ, None, 0, needle, true)
}

#[inline]
fn new_shr(s: &str, typ: TokenType) -> RegexToken{
    RegexToken::new(s, typ, None, 0, true)
}


#[inline]
fn new_cap(s: &str, typ: TokenType, i: usize, probe: &'static str) -> RegexToken{
    RegexToken::new_nd(s, typ, Some(i), 0, probe, false)
}

// TODO: start with ^
pub fn sql_regex() -> Vec<RegexToken> {
    // let literal = create_string_regex(vec!["''", r#""""#, "``"]).unwrap();
    vec![
        new_rt(r"(--|# )\+.*?(\r\n|\r|\n|$)", TokenType::CommentSingleHint),
        new_rt(r"/\*\+[\s\S]*?\*/", TokenType::CommentMultilineHint),

        new_rt(r"(--|# ).*?(\r\n|\r|\n|$)", TokenType::CommentSingle),
        new_rt(r"/\*[\s\S]*?\*/", TokenType::CommentMultiline),

        new_rt(r"(\r\n|\r|\n)", TokenType::Newline),
        new_rt(r"\s+?", TokenType::Whitespace),

        new_rt(r":=", TokenType::Assignment),
        new_rt(r"::", TokenType::Punctuation),

        new_rt(r"\*", TokenType::Wildcard),

        new_rt(r"`(``|[^`])*`", TokenType::Name),
        new_rt(r"´(´´|[^´])*´", TokenType::Name),
        // RegexToken::new_reg(literal, TokenType::Literal),

        new_rt(r"\?", TokenType::NamePlaceholder),
        new_rt(r"%(\(\w+\))?s", TokenType::NamePlaceholder),
        new_rt(r"^[$:?]\w+", TokenType::NamePlaceholder),

        new_rt(r"\\\w+", TokenType::Command),
        new_rt(r"(?i)(NOT\s+)?IN\b", TokenType::OperatorComparison),

        new_shr(r"(?i)^(CASE|IN|VALUES|USING|FROM|AS)\b", TokenType::Keyword),

        new_rt(r"(?i)(@|##|#)[A-ZÀ-Ü]\w+", TokenType::Name), // max name length is 64
        new_cap(r"(?i)([A-ZÀ-Ü]\w*)(?:\s*\.)", TokenType::Name, 1, "."),
        // FIXME: backword match  .name
        RegexToken::new(r"(?i:\.)(?i)([A-ZÀ-Ü]\w*)", TokenType::Name, Some(1), 1, false),
        new_cap(r"(?i)([A-ZÀ-Ü]\w*)(?:\()", TokenType::Name, 1, "("),

        new_rt(r"-?0x[\dA-Fa-f]+", TokenType::NumberHexadecimal),
        new_rt(r"-?\d+(\.\d+)?[Ee]-?\d+", TokenType::NumberFloat),
        new_nd(r"-?(\d+(\.\d*)|\.\d+)", TokenType::NumberFloat, "."),
        new_rt(r"(-\s*)?[0-9]+", TokenType::NumberInteger),

        new_rt(r"'(''|\\\\|\\'|[^'])*'", TokenType::StringSingle),
        new_rt(r#""(""|\\\\|\\"|[^"])*""#, TokenType::StringSymbol),
        new_rt(r#"(""|".*?[^\\]")"#, TokenType::StringSymbol),
        // new_rt(r#"(?:[^\w\])])(\[[^\]\[]+\])"#, TokenType::Name),

        new_nd(r"(?i)((LEFT\s+|RIGHT\s+|FULL\s+)?(INNER\s+|OUTER\s+|STRAIGHT\s+)?|(CROSS\s+|NATURAL\s+)?)?JOIN\b", TokenType::Keyword, "join"),
        new_rt(r"(?i)END(\s+IF|\s+LOOP|\s+WHILE)?\b", TokenType::Keyword),
        new_rt(r"(?i)NOT\s+NULL\b", TokenType::Keyword),
        new_rt(r"(?i)NULLS\s+(FIRST|LAST)\b", TokenType::Keyword),
        new_rt(r"(?i)UNION\s+ALL\b", TokenType::Keyword),
        new_rt(r"(?i)CREATE(\s+OR\s+REPLACE)?\b", TokenType::KeywordDDL),
        new_rt(r"(?i)DOUBLE\s+PRECISION\b", TokenType::NameBuiltin),
        new_sh(r"(?i)^(GROUP\s+BY\b)", TokenType::Keyword, " by"),
        new_sh(r"(?i)^(ORDER\s+BY\b)", TokenType::Keyword, " by"),
        new_rt(r"(?i)^(HANDLER\s+FOR\b)", TokenType::Keyword),
        new_rt(r"(?i)(LATERAL\s+VIEW\s+)(EXPLODE|INLINE|PARSE_URL_TUPLE|POSEXPLODE|STACK)\b", TokenType::Keyword),
        new_rt(r"(?i)(AT|WITH')\s+TIME\s+ZONE\s+'[^']+'", TokenType::KeywordTZCast),
        new_rt(r"(?i)(NOT\s+)?(LIKE|ILIKE|RLIKE)\b", TokenType::OperatorComparison),
        new_rt(r"(?i)[0-9_A-Z][_$#\w]{0,26}", TokenType::KeywordRaw), // min length keyword: as, max length keyword: TRANSACTIONS_ROLLED_BACK TODO: move to special case match with trie
        new_rt(r"[;:()\[\],\.]", TokenType::Punctuation),
        new_rt(r"[<>=~!]+", TokenType::OperatorComparison),
        new_rt(r"[+/@#%^&|^-]+", TokenType::Operator)
    ]
}

// TODO: hash map
pub fn is_keyword(k: &str) -> TokenType {
    let keyword = k.to_uppercase();
    match keyword.as_ref() {
        // KEYWORDS_COMMON
        "SELECT" | "INSERT" | "DELETE" | "UPDATE" | "UPSERT" |"REPLACE" |  "MERGE" | "DROP" | "CREATE" | "ALTER" => TokenType::KeywordDML,
        "WHERE" |"FROM" |"INNER" |"JOIN" |"STRAIGHT_JOIN" |"AND" |"OR" |"LIKE" | "ILIKE" | "RLIKE" |"ON" |"IN" |"SET" => TokenType::Keyword,
        "BY" | "GROUP" |"ORDER" |"LEFT" |"OUTER" |"FULL" => TokenType::Keyword,
        "IF" |"END" |"THEN" |"LOOP" |"AS" |"ELSE" |"FOR" |"WHILE" => TokenType::Keyword,
        "CASE" | "WHEN" | "MIN" | "MAX" | "DISTINCT" => TokenType::Keyword,
        // PostgreSQL
        "CONFLICT" | "WINDOW" | "PARTITION" | "OVER" | "PERFORM" | "NOTICE" | "PLPGSQL" | "INHERIT" | "INDEXES" | "ON_ERROR_STOP" => TokenType::Keyword,
        "BYTEA" | "BIGSERIAL" | "BIT VARYING" | "BOX"  => TokenType::Keyword,
        "CHARACTER" | "CHARACTER VARYING" | "CIDR" | "CIRCLE" => TokenType::Keyword,
        "DOUBLE PRECISION" | "INET" | "JSON" | "JSONB" | "LINE" | "LSEG" | "MACADDR" | "MONEY" => TokenType::Keyword,
        "PATH" | "PG_LSN" | "POINT" | "POLYGON" | "SMALLSERIAL" | "TSQUERY" | "TSVECTOR" | "TXID_SNAPSHOT" | "UUID" | "XML" => TokenType::Keyword,
        // KEYWORDS
        "ABORT" | "ABS" | "ABSOLUTE" | "ACCESS" | "ADA" | "ADD" | "ADMIN" | "AFTER" | "AGGREGATE" | "ALIAS" | "ALL" | "ALLOCATE" | "ANALYSE" | "ANALYZE" | "ANY" | "ARRAYLEN" | "ARE" | "ASENSITIVE" | "ASSERTION" | "ASSIGNMENT" | "ASYMMETRIC" | "AT" | "ATOMIC" | "AUDIT" | "AUTHORIZATION" | "AUTO_INCREMENT" | "AVG" => TokenType::Keyword,
        "ASC" | "DESC" => TokenType::KeywordOrder,
        "BACKWARD" |"BEFORE" |"BEGIN" |"BETWEEN" |"BITVAR" |"BIT_LENGTH" |"BOTH" |"BREADTH" => TokenType::Keyword,
        "CACHE" | "CALL" | "CALLED" | "CARDINALITY" | "CASCADE" | "CASCADED" | "CAST" | "CATALOG" | "CATALOG_NAME" | "CHAIN" | "CHARACTERISTICS" | "CHARACTER_LENGTH" | "CHARACTER_SET_CATALOG" | "CHARACTER_SET_NAME" | "CHARACTER_SET_SCHEMA" | "CHAR_LENGTH" | "CHARSET" | "CHECK" | "CHECKED" | "CHECKPOINT" | "CLASS" | "CLASS_ORIGIN" => TokenType::Keyword,
        "CLOB" | "CLOSE" | "CLUSTER" | "COALESCE" | "COBOL" | "COLLATE" | "COLLATION" | "COLLATION_CATALOG" | "COLLATION_NAME" | "COLLATION_SCHEMA" | "COLLECT" | "COLUMN" | "COLUMN_NAME" | "COMPRESS" | "COMMAND_FUNCTION" | "COMMAND_FUNCTION_CODE" | "COMMENT" | "COMMITTED" | "COMPLETION" => TokenType::Keyword,
        "COMMIT" => TokenType::KeywordDML,
        "CONCURRENTLY" | "CONDITION_NUMBER" | "CONNECT" | "CONNECTION" | "CONNECTION_NAME" | "CONSTRAINT" | "CONSTRAINTS" | "CONSTRAINT_CATALOG" | "CONSTRAINT_NAME" | "CONSTRAINT_SCHEMA" | "CONSTRUCTOR" | "CONTAINS" | "CONTINUE" | "CONVERSION" | "CONVERT" | "COPY" | "CORRESPONDING" | "COUNT" => TokenType::Keyword,
        "CREATEDB" | "CREATEUSER" | "CROSS" | "CUBE" | "CURRENT" | "CURRENT_DATE" | "CURRENT_PATH" | "CURRENT_ROLE" | "CURRENT_TIME" | "CURRENT_TIMESTAMP" | "CURRENT_USER" | "CURSOR" | "CURSOR_NAME" | "CYCLE" => TokenType::Keyword,
        "DATA" | "DATABASE" | "DATETIME_INTERVAL_CODE" | "DATETIME_INTERVAL_PRECISION" | "DAY" | "DEALLOCATE" | "DECLARE" | "DEFAULT" | "DEFAULTS" | "DEFERRABLE" | "DEFERRED" | "DEFINED" | "DEFINER" | "DELIMITER" => TokenType::Keyword,
        "DELIMITERS" | "DEREF"| "DESCRIBE" | "DESCRIPTOR" | "DESTROY" | "DESTRUCTOR" | "DETERMINISTIC" | "DIAGNOSTICS" | "DICTIONARY" | "DISABLE" | "DISCONNECT" | "DISPATCH" | "DO" | "DOMAIN" | "DYNAMIC" | "DYNAMIC_FUNCTION" | "DYNAMIC_FUNCTION_CODE" => TokenType::Keyword,
        "EACH" | "ENABLE" | "ENCODING" | "ENCRYPTED" | "ENGINE" | "EQUALS" | "ESCAPE" | "EVERY" | "EXCEPT" | "EXCEPTION" | "EXCLUDING" | "EXCLUSIVE" | "EXEC" | "EXECUTE" | "EXISTING" | "EXISTS" | "EXPLAIN" | "EXTERNAL" | "EXTRACT" => TokenType::Keyword,
        "FALSE" | "FETCH" | "FILE" | "FINAL" | "FIRST" | "FORCE" | "FOREACH" | "FOREIGN" | "FORTRAN" | "FORWARD" | "FOUND" | "FREE" | "FREEZE" | "FUNCTION" => TokenType::Keyword,
        "GENERAL" | "GENERATED" | "GET" | "GLOBAL" | "GO" | "GOTO" | "GRANT" | "GRANTED" | "GROUPING" => TokenType::Keyword,
        "HAVING" | "HIERARCHY" | "HOLD" | "HOUR" | "HOST" => TokenType::Keyword,
        "IMPLEMENTATION" | "IMPLICIT" | "INCLUDING" | "INCREMENT" | "INDEX" => TokenType::Keyword,
        "INDITCATOR" | "INFIX" | "INHERITS" | "INITIAL" | "INITIALIZE" | "INITIALLY" | "INOUT" | "INPUT" | "INSENSITIVE" | "INSTANTIABLE" | "INSTEAD" | "INTERSECT" | "INTO" | "INVOKER" | "IS" | "ISNULL" | "ISOLATION" | "ITERATE" => TokenType::Keyword,
        "KEY" | "KEY_MEMBER" | "KEY_TYPE" => TokenType::Keyword,
        "LANCOMPILER" | "LANGUAGE" | "LARGE" | "LAST" | "LATERAL" | "LEADING" | "LENGTH" | "LESS" | "LEVEL" | "LIMIT" | "LISTEN" | "LOAD" | "LOCAL" | "LOCALTIME" | "LOCALTIMESTAMP" | "LOCATION" | "LOCATOR" | "LOCK" | "LOWER" => TokenType::Keyword,
        "MAP" | "MATCH" | "MAXEXTENTS" | "MAXVALUE" | "MESSAGE_LENGTH" | "MESSAGE_OCTET_LENGTH" | "MESSAGE_TEXT" | "METHOD" | "MINUTE" | "MINUS" | "MINVALUE" | "MOD" | "MODE" | "MODIFIES" | "MODIFY" | "MONTH" | "MORE" | "MOVE" | "MUMPS" => TokenType::Keyword,
        "NAMES" | "NATIONAL" | "NATURAL" | "NCHAR" | "NCLOB" | "NEW" | "NEXT" | "NO" | "NOAUDIT" | "NOCOMPRESS" | "NOCREATEDB" | "NOCREATEUSER" | "NONE" | "NOT" | "NOTFOUND" | "NOTHING" | "NOTIFY" | "NOTNULL" | "NOWAIT" | "NULL" | "NULLABLE" | "NULLIF" => TokenType::Keyword,
        "OBJECT" | "OCTET_LENGTH" | "OF" | "OFF" | "OFFLINE" | "OFFSET" | "OIDS" | "OLD" | "ONLINE" | "ONLY" | "OPEN" | "OPERATION" | "OPERATOR" | "OPTION" | "OPTIONS" | "ORDINALITY" | "OUT" | "OUTPUT" | "OVERLAPS" | "OVERLAY" | "OVERRIDING" | "OWNER" => TokenType::Keyword,
        "QUARTER" => TokenType::Keyword,
        "PAD" | "PARAMETER" | "PARAMETERS" | "PARAMETER_MODE" | "PARAMETER_NAME" | "PARAMETER_ORDINAL_POSITION" | "PARAMETER_SPECIFIC_CATALOG" | "PARAMETER_SPECIFIC_NAME" | "PARAMETER_SPECIFIC_SCHEMA" | "PARTIAL" | "PASCAL" => TokenType::Keyword,
        "PCTFREE" | "PENDANT" | "PLACING" | "PLI" | "POSITION" | "POSTFIX" | "PRECISION" | "PREFIX" | "PREORDER" | "PREPARE" | "PRESERVE" | "PRIMARY" | "PRIOR" | "PRIVILEGES" | "PROCEDURAL" | "PROCEDURE" | "PUBLIC" => TokenType::Keyword,
        "RAISE" | "RAW" | "READ" | "READS" | "RECHECK" | "RECURSIVE" | "REF" | "REFERENCES" | "REFERENCING" | "REINDEX" | "RELATIVE" | "RENAME" | "REPEATABLE" | "RESET" | "RESOURCE" | "RESTART" | "RESTRICT" | "RESULT" | "RETURN" | "RETURNED_LENGTH" | "RETURNED_OCTET_LENGTH" | "RETURNED_SQLSTATE" | "RETURNING" | "RETURNS" | "REVOKE" | "RIGHT" | "ROLE" => TokenType::Keyword,
        "ROLLBACK" => TokenType::KeywordDML,
        "ROLLUP" | "ROUTINE" | "ROUTINE_CATALOG" | "ROUTINE_NAME" | "ROUTINE_SCHEMA" | "ROW" | "ROWS" | "ROW_COUNT" | "RULE" => TokenType::Keyword,
        "SAVE_POINT" | "SCALE" | "SCHEMA" | "SCHEMA_NAME" | "SCOPE" | "SCROLL" | "SEARCH" | "SECOND" | "SECURITY" | "SELF" | "SENSITIVE" | "SEQUENCE" | "SERIALIZABLE" | "SERVER_NAME" | "SESSION" | "SESSION_USER" | "SETOF" => TokenType::Keyword,
        "SETS" | "SHARE" | "SHOW" | "SIMILAR" | "SIMPLE" | "SIZE" | "SOME" | "SOURCE" | "SPACE" | "SPECIFIC" | "SPECIFICTYPE" | "SPECIFIC_NAME" | "SQL" | "SQLBUF" | "SQLCODE" | "SQLERROR" | "SQLEXCEPTION" | "SQLSTATE" | "SQLWARNING" | "STABLE" => TokenType::Keyword,
        "START" => TokenType::KeywordDML,
        "STATEMENT" | "STATIC" | "STATISTICS" | "STDIN" | "STDOUT" | "STORAGE" | "STRICT" | "STRUCTURE" | "STYPE" | "SUBCLASS_ORIGIN" | "SUBLIST" | "SUBSTRING" | "SUCCESSFUL" | "SUM" | "SYMMETRIC" | "SYNONYM" | "SYSID" | "SYSTEM" | "SYSTEM_USER" => TokenType::Keyword,
        "TABLE" | "TABLE_NAME" | "TEMP" | "TEMPLATE" | "TEMPORARY" | "TERMINATE" | "THAN" | "TIMESTAMP" | "TIMEZONE_HOUR" | "TIMEZONE_MINUTE" | "TO" | "TOAST" | "TRAILING" | "TRANSATION" | "TRANSACTIONS_COMMITTED" | "TRANSACTIONS_ROLLED_BACK" => TokenType::Keyword,
        "TRANSATION_ACTIVE" | "TRANSFORM" | "TRANSFORMS" | "TRANSLATE" | "TRANSLATION" | "TREAT" | "TRIGGER" | "TRIGGER_CATALOG" | "TRIGGER_NAME" | "TRIGGER_SCHEMA" | "TRIM" | "TRUE" | "TRUNCATE" | "TRUSTED" | "TYPE" => TokenType::Keyword,
        "UID" | "UNCOMMITTED" | "UNDER" | "UNENCRYPTED" | "UNION" | "UNIQUE" | "UNKNOWN" | "UNLISTEN" | "UNNAMED" | "UNNEST" | "UNTIL" | "UPPER" | "USAGE" | "USE" | "USER" | "USER_DEFINED_TYPE_CATALOG" | "USER_DEFINED_TYPE_NAME" | "USER_DEFINED_TYPE_SCHEMA" | "USING" => TokenType::Keyword,
        "VACUUM" | "VALID" | "VALIDATE" | "VALIDATOR" | "VALUES" | "VARIABLE" | "VERBOSE" | "VERSION" | "VIEW" | "VOLATILE" => TokenType::Keyword,
        "WEEK" | "WHENEVER" => TokenType::Keyword,
        "WITH" => TokenType::KeywordCTE,
        "WITHOUT" | "WORK" | "WRITE"  => TokenType::Keyword,
        "YEAR" => TokenType::Keyword,
        "ZONE" => TokenType::Keyword,
        "ARRAY" | "BIGINT" | "BINARY" | "BIT" | "BLOB" | "BOOLEAN" | "CHAR" | "DATE" | "DEC" | "DECIMAL" | "FILE_TYPE" | "FLOAT" | "INT" | "INT8" | "INTEGER" | "INTERVAL" | "LONG" | "NATURALN" | "NVARCHAR" | "NUMBER" | "NUMERIC" | "PLS_INTEGER" | "POSITIVE" | "POSITIVEN" | "REAL" | "ROWID" | "ROWLABEL" => TokenType::NameBuiltin,
        "ROWNUM" | "SERIAL" | "SERIAL8" | "SIGNED" | "SIGNTYPE" | "SIMPLE_DOUBLE" | "SIMPLE_FLOAT" | "SIMPLE_INTEGER" | "SMALLINT" | "SYS_REFCURSOR" | "SYSDATE" | "TEXT" | "TINYINT" | "UNSIGNED" | "UROWID" | "UTL_FILE" | "VARCHAR" | "VARCHAR2" | "VARYING" => TokenType::NameBuiltin,
        _ => TokenType::Name,
    }
}

const KEYWORDS_DML: [&str; 13] = ["SELECT" , "INSERT" , "DELETE" , "UPDATE" , "UPSERT" , "REPLACE" ,  "MERGE" , "DROP" , "CREATE" , "ALTER", "COMMIT", "ROLLBACK", "START"];
const KEYWORDS_ORDER: [&str; 2] = ["ASC" , "DESC"];
const NAME_BUILTIN: [&str; 46] = ["ARRAY" , "BIGINT" , "BINARY" , "BIT" , "BLOB" , "BOOLEAN" , "CHAR" , "DATE" , "DEC" , "DECIMAL" , "FILE_TYPE" , "FLOAT" , "INT" , "INT8" , "INTEGER" , "INTERVAL" , "LONG" , "NATURALN" , "NVARCHAR" , "NUMBER" , "NUMERIC" , "PLS_INTEGER" , "POSITIVE" , "POSITIVEN" , "REAL" , "ROWID" , "ROWLABEL",
"ROWNUM" , "SERIAL" , "SERIAL8" , "SIGNED" , "SIGNTYPE" , "SIMPLE_DOUBLE" , "SIMPLE_FLOAT" , "SIMPLE_INTEGER" , "SMALLINT" , "SYS_REFCURSOR" , "SYSDATE" , "TEXT" , "TINYINT" , "UNSIGNED" , "UROWID" , "UTL_FILE" , "VARCHAR" , "VARCHAR2" , "VARYING"];
const KEYWORDS_CTE:[&str; 1] = ["WITH"];

const KEYWORDS: [&str; 550] = [
    "WHERE" ,"FROM" ,"INNER" ,"JOIN" ,"STRAIGHT_JOIN" ,"AND" ,"OR" ,"LIKE" , "ILIKE" , "RLIKE" ,"ON" ,"IN" ,"SET",
    "BY" , "GROUP" ,"ORDER" ,"LEFT" ,"OUTER" ,"FULL",
    "IF" ,"END" ,"THEN" ,"LOOP" ,"AS" ,"ELSE" ,"FOR" ,"WHILE",
    "CASE" , "WHEN" , "MIN" , "MAX" , "DISTINCT",
    "CONFLICT" , "WINDOW" , "PARTITION" , "OVER" , "PERFORM" , "NOTICE" , "PLPGSQL" , "INHERIT" , "INDEXES" , "ON_ERROR_STOP",
    "BYTEA" , "BIGSERIAL" , "BIT VARYING" , "BOX" ,
    "CHARACTER" , "CHARACTER VARYING" , "CIDR" , "CIRCLE",
    "DOUBLE PRECISION" , "INET" , "JSON" , "JSONB" , "LINE" , "LSEG" , "MACADDR" , "MONEY",
    "PATH" , "PG_LSN" , "POINT" , "POLYGON" , "SMALLSERIAL" , "TSQUERY" , "TSVECTOR" , "TXID_SNAPSHOT" , "UUID" , "XML",
    "ABORT" , "ABS" , "ABSOLUTE" , "ACCESS" , "ADA" , "ADD" , "ADMIN" , "AFTER" , "AGGREGATE" , "ALIAS" , "ALL" , "ALLOCATE" , "ANALYSE" , "ANALYZE" , "ANY" , "ARRAYLEN" , "ARE" , "ASENSITIVE" , "ASSERTION" , "ASSIGNMENT" , "ASYMMETRIC" , "AT" , "ATOMIC" , "AUDIT" , "AUTHORIZATION" , "AUTO_INCREMENT" , "AVG",
    "BACKWARD" ,"BEFORE" ,"BEGIN" ,"BETWEEN" ,"BITVAR" ,"BIT_LENGTH" ,"BOTH" ,"BREADTH",
    "CACHE" , "CALL" , "CALLED" , "CARDINALITY" , "CASCADE" , "CASCADED" , "CAST" , "CATALOG" , "CATALOG_NAME" , "CHAIN" , "CHARACTERISTICS" , "CHARACTER_LENGTH" , "CHARACTER_SET_CATALOG" , "CHARACTER_SET_NAME" , "CHARACTER_SET_SCHEMA" , "CHAR_LENGTH" , "CHARSET" , "CHECK" , "CHECKED" , "CHECKPOINT" , "CLASS" , "CLASS_ORIGIN",
    "CLOB" , "CLOSE" , "CLUSTER" , "COALESCE" , "COBOL" , "COLLATE" , "COLLATION" , "COLLATION_CATALOG" , "COLLATION_NAME" , "COLLATION_SCHEMA" , "COLLECT" , "COLUMN" , "COLUMN_NAME" , "COMPRESS" , "COMMAND_FUNCTION" , "COMMAND_FUNCTION_CODE" , "COMMENT" , "COMMITTED" , "COMPLETION",
    // "COMMIT",
    "CONCURRENTLY" , "CONDITION_NUMBER" , "CONNECT" , "CONNECTION" , "CONNECTION_NAME" , "CONSTRAINT" , "CONSTRAINTS" , "CONSTRAINT_CATALOG" , "CONSTRAINT_NAME" , "CONSTRAINT_SCHEMA" , "CONSTRUCTOR" , "CONTAINS" , "CONTINUE" , "CONVERSION" , "CONVERT" , "COPY" , "CORRESPONDING" , "COUNT",
    "CREATEDB" , "CREATEUSER" , "CROSS" , "CUBE" , "CURRENT" , "CURRENT_DATE" , "CURRENT_PATH" , "CURRENT_ROLE" , "CURRENT_TIME" , "CURRENT_TIMESTAMP" , "CURRENT_USER" , "CURSOR" , "CURSOR_NAME" , "CYCLE",
    "DATA" , "DATABASE" , "DATETIME_INTERVAL_CODE" , "DATETIME_INTERVAL_PRECISION" , "DAY" , "DEALLOCATE" , "DECLARE" , "DEFAULT" , "DEFAULTS" , "DEFERRABLE" , "DEFERRED" , "DEFINED" , "DEFINER" , "DELIMITER",
    "DELIMITERS" , "DEREF", "DESCRIBE" , "DESCRIPTOR" , "DESTROY" , "DESTRUCTOR" , "DETERMINISTIC" , "DIAGNOSTICS" , "DICTIONARY" , "DISABLE" , "DISCONNECT" , "DISPATCH" , "DO" , "DOMAIN" , "DYNAMIC" , "DYNAMIC_FUNCTION" , "DYNAMIC_FUNCTION_CODE",
    "EACH" , "ENABLE" , "ENCODING" , "ENCRYPTED" , "ENGINE" , "EQUALS" , "ESCAPE" , "EVERY" , "EXCEPT" , "EXCEPTION" , "EXCLUDING" , "EXCLUSIVE" , "EXEC" , "EXECUTE" , "EXISTING" , "EXISTS" , "EXPLAIN" , "EXTERNAL" , "EXTRACT",
    "FALSE" , "FETCH" , "FILE" , "FINAL" , "FIRST" , "FORCE" , "FOREACH" , "FOREIGN" , "FORTRAN" , "FORWARD" , "FOUND" , "FREE" , "FREEZE" , "FUNCTION",
    "GENERAL" , "GENERATED" , "GET" , "GLOBAL" , "GO" , "GOTO" , "GRANT" , "GRANTED" , "GROUPING",
    "HAVING" , "HIERARCHY" , "HOLD" , "HOUR" , "HOST",
    "IMPLEMENTATION" , "IMPLICIT" , "INCLUDING" , "INCREMENT" , "INDEX",
    "INDITCATOR" , "INFIX" , "INHERITS" , "INITIAL" , "INITIALIZE" , "INITIALLY" , "INOUT" , "INPUT" , "INSENSITIVE" , "INSTANTIABLE" , "INSTEAD" , "INTERSECT" , "INTO" , "INVOKER" , "IS" , "ISNULL" , "ISOLATION" , "ITERATE",
    "KEY" , "KEY_MEMBER" , "KEY_TYPE",
    "LANCOMPILER" , "LANGUAGE" , "LARGE" , "LAST" , "LATERAL" , "LEADING" , "LENGTH" , "LESS" , "LEVEL" , "LIMIT" , "LISTEN" , "LOAD" , "LOCAL" , "LOCALTIME" , "LOCALTIMESTAMP" , "LOCATION" , "LOCATOR" , "LOCK" , "LOWER",
    "MAP" , "MATCH" , "MAXEXTENTS" , "MAXVALUE" , "MESSAGE_LENGTH" , "MESSAGE_OCTET_LENGTH" , "MESSAGE_TEXT" , "METHOD" , "MINUTE" , "MINUS" , "MINVALUE" , "MOD" , "MODE" , "MODIFIES" , "MODIFY" , "MONTH" , "MORE" , "MOVE" , "MUMPS",
    "NAMES" , "NATIONAL" , "NATURAL" , "NCHAR" , "NCLOB" , "NEW" , "NEXT" , "NO" , "NOAUDIT" , "NOCOMPRESS" , "NOCREATEDB" , "NOCREATEUSER" , "NONE" , "NOT" , "NOTFOUND" , "NOTHING" , "NOTIFY" , "NOTNULL" , "NOWAIT" , "NULL" , "NULLABLE" , "NULLIF",
    "OBJECT" , "OCTET_LENGTH" , "OF" , "OFF" , "OFFLINE" , "OFFSET" , "OIDS" , "OLD" , "ONLINE" , "ONLY" , "OPEN" , "OPERATION" , "OPERATOR" , "OPTION" , "OPTIONS" , "ORDINALITY" , "OUT" , "OUTPUT" , "OVERLAPS" , "OVERLAY" , "OVERRIDING" , "OWNER",
    "QUARTER" ,
    "PAD" , "PARAMETER" , "PARAMETERS" , "PARAMETER_MODE" , "PARAMETER_NAME" , "PARAMETER_ORDINAL_POSITION" , "PARAMETER_SPECIFIC_CATALOG" , "PARAMETER_SPECIFIC_NAME" , "PARAMETER_SPECIFIC_SCHEMA" , "PARTIAL" , "PASCAL" ,
    "PCTFREE" , "PENDANT" , "PLACING" , "PLI" , "POSITION" , "POSTFIX" , "PRECISION" , "PREFIX" , "PREORDER" , "PREPARE" , "PRESERVE" , "PRIMARY" , "PRIOR" , "PRIVILEGES" , "PROCEDURAL" , "PROCEDURE" , "PUBLIC",
    "RAISE" , "RAW" , "READ" , "READS" , "RECHECK" , "RECURSIVE" , "REF" , "REFERENCES" , "REFERENCING" , "REINDEX" , "RELATIVE" , "RENAME" , "REPEATABLE" , "RESET" , "RESOURCE" , "RESTART" , "RESTRICT" , "RESULT" , "RETURN" , "RETURNED_LENGTH" , "RETURNED_OCTET_LENGTH" , "RETURNED_SQLSTATE" , "RETURNING" , "RETURNS" , "REVOKE" , "RIGHT" , "ROLE" ,
    // "ROLLBACK" ,
    "ROLLUP" , "ROUTINE" , "ROUTINE_CATALOG" , "ROUTINE_NAME" , "ROUTINE_SCHEMA" , "ROW" , "ROWS" , "ROW_COUNT" , "RULE" ,
    "SAVE_POINT" , "SCALE" , "SCHEMA" , "SCHEMA_NAME" , "SCOPE" , "SCROLL" , "SEARCH" , "SECOND" , "SECURITY" , "SELF" , "SENSITIVE" , "SEQUENCE" , "SERIALIZABLE" , "SERVER_NAME" , "SESSION" , "SESSION_USER" , "SETOF",
    "SETS" , "SHARE" , "SHOW" , "SIMILAR" , "SIMPLE" , "SIZE" , "SOME" , "SOURCE" , "SPACE" , "SPECIFIC" , "SPECIFICTYPE" , "SPECIFIC_NAME" , "SQL" , "SQLBUF" , "SQLCODE" , "SQLERROR" , "SQLEXCEPTION" , "SQLSTATE" , "SQLWARNING" , "STABLE" ,
    // "START",
    "STATEMENT" , "STATIC" , "STATISTICS" , "STDIN" , "STDOUT" , "STORAGE" , "STRICT" , "STRUCTURE" , "STYPE" , "SUBCLASS_ORIGIN" , "SUBLIST" , "SUBSTRING" , "SUCCESSFUL" , "SUM" , "SYMMETRIC" , "SYNONYM" , "SYSID" , "SYSTEM" , "SYSTEM_USER",
    "TABLE" , "TABLE_NAME" , "TEMP" , "TEMPLATE" , "TEMPORARY" , "TERMINATE" , "THAN" , "TIMESTAMP" , "TIMEZONE_HOUR" , "TIMEZONE_MINUTE" , "TO" , "TOAST" , "TRAILING" , "TRANSATION" , "TRANSACTIONS_COMMITTED" , "TRANSACTIONS_ROLLED_BACK",
    "TRANSATION_ACTIVE" , "TRANSFORM" , "TRANSFORMS" , "TRANSLATE" , "TRANSLATION" , "TREAT" , "TRIGGER" , "TRIGGER_CATALOG" , "TRIGGER_NAME" , "TRIGGER_SCHEMA" , "TRIM" , "TRUE" , "TRUNCATE" , "TRUSTED" , "TYPE",
    "UID" , "UNCOMMITTED" , "UNDER" , "UNENCRYPTED" , "UNION" , "UNIQUE" , "UNKNOWN" , "UNLISTEN" , "UNNAMED" , "UNNEST" , "UNTIL" , "UPPER" , "USAGE" , "USE" , "USER" , "USER_DEFINED_TYPE_CATALOG" , "USER_DEFINED_TYPE_NAME" , "USER_DEFINED_TYPE_SCHEMA" , "USING" ,
    "VACUUM" , "VALID" , "VALIDATE" , "VALIDATOR" , "VALUES" , "VARIABLE" , "VERBOSE" , "VERSION" , "VIEW" , "VOLATILE",
    "WEEK" , "WHENEVER",
    // "WITH" ,
    "WITHOUT" , "WORK" , "WRITE",
    "YEAR",
    "ZONE",
];

pub fn init_trie() -> Trie {
    let mut t = Trie::default();
    for keyword in KEYWORDS { t.insert_token(keyword, TokenType::Keyword) }
    for keyword in KEYWORDS_ORDER { t.insert_token(keyword, TokenType::KeywordOrder) }
    for keyword in KEYWORDS_DML { t.insert_token(keyword, TokenType::KeywordDML) }
    for keyword in NAME_BUILTIN { t.insert_token(keyword, TokenType::NameBuiltin) }
    for keyword in KEYWORDS_CTE { t.insert_token(keyword, TokenType::KeywordCTE) }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_sql_regex() {
        let regs = sql_regex();
        assert!(regs.len() > 0)
    }

    #[test]
    fn test_non_capturing_group() {
        let reg = Regex::new(r"([A-ZÀ-Ü]\w*)(?:\()").unwrap();
        let c = reg.captures("MAX(price)").unwrap();
        assert_eq!(c.get(1).map(|m| m.as_str()), Some("MAX"));
        let r = reg.find("MAX(price)").unwrap();
        println!("find {:?} {}", r, r.as_str());
       
        let reg = Regex::new(r"(?i)([A-ZÀ-Ü]\w*)(?:\s*\.)").unwrap();
        let r = reg.find("tab.col").unwrap();
        println!("{:?} {}", r, r.as_str());
    }

    #[test]
    fn test_shortest_match() {
        let reg = Regex::new(r"(?i)ORDER\s+BY\b").unwrap();
        let sql = "order by id";
        let result = reg.shortest_match(sql);
        let pos = result.unwrap();
        let s = &sql[0..pos];
        assert_eq!(s, "order by");
    }

    #[test]
    fn test_non_capturing_group1() {
        let reg = Regex::new(r"(?:\.)([A-ZÀ-Ü]\w*)").unwrap();
        let c = reg.captures(".Orders").unwrap();
        println!("{:?}", c);
        assert_eq!(c.get(1).map(|m| m.as_str()), Some("Orders"))
    }

    #[test]
    fn test_slow_regex() {
        let reg = Regex::new(r"(?i)([A-ZÀ-Ü]\w*)(?:\s*\.)").unwrap();
        let now = Instant::now();
        let c = reg.captures("t.col").unwrap();
        println!("captures {}", now.elapsed().as_micros());
        println!("{:?}", c);
        let reg = Regex::new(r"(?i)[A-ZÀ-Ü]\w*(?:\s*\.)").unwrap();
        let s = "t.col";
        let now = Instant::now();
        let c = reg.find(s).unwrap();
        println!("find {}", now.elapsed().as_micros());
        println!("{:?}", &s[c.start()..c.end()]);
    }
}