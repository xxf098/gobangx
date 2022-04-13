use super::{Component, DrawableComponent, EventState};
use crate::components::help_info::HelpInfo;
use crate::event::Key;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct TableValueComponent {
    value: String,
}

impl TableValueComponent {
    pub fn _new(value: String) -> Self {
        Self { value }
    }
}

impl DrawableComponent for TableValueComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let paragraph = Paragraph::new(self.value.clone())
            .block(Block::default().borders(Borders::BOTTOM))
            .style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            });
        f.render_widget(paragraph, area);
        Ok(())
    }
}

impl Component for TableValueComponent {
    fn helps(&self, _out: &mut Vec<HelpInfo>) {}

    fn event(&mut self, _key: &[Key]) -> Result<EventState> {
        todo!("scroll");
    }
}
