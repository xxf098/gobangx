use std::convert::TryFrom;
use tui::text::{Text, Spans, Span};
use tui::style::{Color, Style};
use crate::config::{Settings, DatabaseType};
use crate::sql::token::{tokenizer::Tokenizer, token_type::TokenType };


// fn is_sep(c: &char) -> bool {
//     c.is_ascii_whitespace() || *c == '\0'
// }

// fn is_quote(c: &char) -> bool {
//     *c == '\'' || *c == '"' || *c == '`'
// }

// pub fn highlight<'a>(input: &str, theme: &'a ThemeConfig) -> Text<'a> {
//     let chars = input.chars().collect::<Vec<_>>();
//     let mut s = String::new();
//     let style_hl = Style::default().fg(theme.color);
//     let style_normal = Style::default().fg(Color::White);
//     let mut spans = vec![];
//     let mut quote_count = 0;
//     for (i, c) in chars.iter().enumerate() {
//         s.push(*c);
//         if is_sep(c) {
//             spans.push(Span::styled(s.clone(), style_normal));
//             s = String::new();
//             continue
//         }
//         if is_quote(c) {
//             quote_count += 1;
//         }
//         if KEYWORDS.iter().find(|k| **k == s.to_uppercase()).is_some() && quote_count % 2 == 0 {
//             if chars.get(i+1).map(|c| is_sep(c)).unwrap_or(true) {
//                 spans.push(Span::styled(s.clone(), style_hl));
//                 s = String::new();
//             }
//         }
//     }
//     if s.len() > 0 {
//         spans.push(Span::styled(s.clone(), style_normal));
//     }
//     Text::from(Spans::from(spans))
// }

// TODO: sql formatter
pub fn highlight_sql<'a>(input: &'a str, theme: &'a Settings, database_type: &DatabaseType) -> Text<'a> {
    let style_hl = Style::default().fg(theme.color);
    let style_normal = Style::default().fg(Color::White);
    let mut spans = vec![];
    let t = Tokenizer::try_from(database_type.clone());
    if t.is_err() {
        spans.push(Span::styled(input, style_normal));
        return Text::from(Spans::from(spans));
    };
    let t = t.unwrap();
    let tokens = t.tokenize(input);
    for token in tokens {
        let span = match token.typ {
            TokenType::ReservedTopLevel | TokenType::Reserved => Span::styled(token.to_string(), style_hl),
            _ => Span::styled(token.to_string(), style_normal)
        };
        spans.push(span);
    };
    Text::from(Spans::from(spans))
}

