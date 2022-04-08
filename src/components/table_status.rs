use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::event::Key;
use anyhow::Result;
use database_tree::Table;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct TableStatusComponent {
    column_count: Option<usize>,
    row_count: Option<usize>,
    table: Option<Table>,
    table_value: Option<String>,
}

impl Default for TableStatusComponent {
    fn default() -> Self {
        Self {
            row_count: None,
            column_count: None,
            table: None,
            table_value: None,
        }
    }
}

impl TableStatusComponent {
    pub fn new(
        row_count: Option<usize>,
        column_count: Option<usize>,
        table_value: Option<String>,
        table: Option<Table>,
    ) -> Self {
        Self {
            row_count,
            column_count,
            table,
            table_value,
        }
    }
}

// TODO:
impl DrawableComponent for TableStatusComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let mut spans = vec![
            Span::from(format!(
                "rows: {}, ",
                self.row_count.map_or("-".to_string(), |c| c.to_string())
            )),
            Span::from(format!(
                "columns: {}, ",
                self.column_count.map_or("-".to_string(), |c| c.to_string())
            )),
        ];
        if let Some(table) = &self.table {
            if let Some(engine) = &table.engine {
                spans.push(Span::from(format!("engine: {}, ", engine.to_string())))
            }
        }
        spans.push(Span::from(format!("{}", self.table_value.as_deref().unwrap_or(""))));
        let status = Paragraph::new(Spans::from(spans))
        .block(Block::default().borders(Borders::TOP).style(if focused {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        }));
        f.render_widget(status, area);
        Ok(())
    }
}

impl Component for TableStatusComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, _key: &[Key]) -> Result<EventState> {
        Ok(EventState::NotConsumed)
    }
}
