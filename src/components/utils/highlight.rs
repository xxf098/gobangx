use tui::text::{Text, Spans, Span};
use tui::style::{Color, Style};
use crate::config::{ThemeConfig};

const KEYWORDS: [&str; 8] = ["SELECT", "FROM", "WHERE", "ORDER", "BY", "LIMIT", "JOIN", "LIKE"]; 

fn is_sep(c: &char) -> bool {
    c.is_ascii_whitespace() || *c == '\0'
}

// FIXME
pub fn highlight<'a>(input: &str, theme: &'a ThemeConfig) -> Text<'a> {
    let chars = input.chars().collect::<Vec<_>>();
    let mut s = String::new();
    let style_hl = Style::default().fg(theme.color);
    let style_normal = Style::default().fg(Color::White);
    let mut spans = vec![];
    for (i, c) in chars.iter().enumerate() {
        s.push(*c);
        if is_sep(c) {
            spans.push(Span::styled(s.clone(), style_normal));
            s = String::new();
            continue
        }
        if KEYWORDS.iter().find(|k| **k == s.to_uppercase()).is_some() {
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

