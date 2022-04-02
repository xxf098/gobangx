use tui::text::{Text, Spans, Span};
use tui::style::{Color, Style};
use crate::config::{ThemeConfig};

const KEYWORDS: [&str; 128] = [
    "ADD",
    "ADD CONSTRAINT",
    "ALTER",
    "ALTER COLUMN",
    "ALTER TABLE",
    "ALL",
    "AND",
    "ANY",
    "AS",
    "ASC",
    "BACKUP DATABASE",
    "BETWEEN",
    "CASE",
    "CHECK",
    "COLUMN",
    "CONSTRAINT",
    "CREATE",
    "CREATE DATABASE",
    "CREATE INDEX",
    "CREATE OR REPLACE VIEW",
    "CREATE TABLE",
    "CREATE PROCEDURE",
    "CREATE UNIQUE INDEX",
    "CREATE VIEW",
    "DATABASE",
    "DEFAULT",
    "DELETE",
    "DESC",
    "DISTINCT",
    "DROP",
    "DROP COLUMN",
    "DROP CONSTRAINT",
    "DROP DATABASE",
    "DROP DEFAULT",
    "DROP INDEX",
    "DROP TABLE",
    "DROP VIEW",
    "EXEC",
    "EXISTS",
    "FOREIGN KEY",
    "FROM",
    "FULL OUTER JOIN",
    "GROUP BY",
    "HAVING",
    "IN",
    "INDEX",
    "INNER JOIN",
    "INSERT INTO",
    "INSERT INTO SELECT",
    "IS NULL",
    "IS NOT NULL",
    "JOIN",
    "LEFT JOIN",
    "LIKE",
    "LIMIT",
    "NOT",
    "NOT NULL",
    "OR",
    "ORDER BY",
    "OUTER JOIN",
    "PRIMARY KEY",
    "PROCEDURE",
    "RIGHT JOIN",
    "ROWNUM",
    "SELECT",
    "SELECT DISTINCT",
    "SELECT INTO",
    "SELECT TOP",
    "SET",
    "TABLE",
    "TOP",
    "TRUNCATE TABLE",
    "UNION",
    "UNION ALL",
    "UNIQUE",
    "UPDATE",
    "VALUES",
    "VIEW",
    "WHERE",
    "PRAGMA",
    "INTEGER",
    "PRIMARY",
    "letCHAR",
    "DATETIME",
    "NULL",
    "REFERENCES",
    "INDEX_LIST",
    "BY",
    "CURRENT_DATE",
    "CURRENT_TIME",
    "EACH",
    "ELSE",
    "ELSEIF",
    "FALSE",
    "FOR",
    "GROUP",
    "IF",
    "INSERT",
    "INTERVAL",
    "INTO",
    "IS",
    "KEY",
    "KEYS",
    "LEFT",
    "MATCH",
    "ON",
    "OPTION",
    "ORDER",
    "OUT",
    "OUTER",
    "REPLACE",
    "TINYINT",
    "RIGHT",
    "THEN",
    "TO",
    "TRUE",
    "WHEN",
    "WITH",
    "UNSIGNED",
    "CASCADE",
    "ENGINE",
    "TEXT",
    "AUTO_INCREMENT",
    "SHOW",
    "BEGIN",
    "END",
    "PRINT",
    "OVERLAPS",
]; 

fn is_sep(c: &char) -> bool {
    c.is_ascii_whitespace() || *c == '\0'
}

fn is_quote(c: &char) -> bool {
    *c == '\'' || *c == '"' || *c == '`'
}

// FIXME
// sql formatter
pub fn highlight<'a>(input: &str, theme: &'a ThemeConfig) -> Text<'a> {
    let chars = input.chars().collect::<Vec<_>>();
    let mut s = String::new();
    let style_hl = Style::default().fg(theme.color);
    let style_normal = Style::default().fg(Color::White);
    let mut spans = vec![];
    let mut quote_count = 0;
    for (i, c) in chars.iter().enumerate() {
        s.push(*c);
        if is_sep(c) {
            spans.push(Span::styled(s.clone(), style_normal));
            s = String::new();
            continue
        }
        if is_quote(c) {
            quote_count += 1;
        }
        if KEYWORDS.iter().find(|k| **k == s.to_uppercase()).is_some() && quote_count % 2 == 0 {
            if chars.get(i+1).map(|c| is_sep(c)).unwrap_or(true) {
                spans.push(Span::styled(s.clone(), style_hl));
                s = String::new();
            }
        }
    }
    if s.len() > 0 {
        spans.push(Span::styled(s.clone(), style_normal));
    }
    Text::from(Spans::from(spans))
}

