use super::{Component, DrawableComponent, EventState};
use crate::components::help_info::{self, CommandInfo};
use crate::config::KeyConfig;
use crate::event::Key;
use anyhow::Result;
use strum_macros::EnumIter;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, Tabs},
    Frame,
};

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Tab {
    Records,
    Properties,
    Sql,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct TabComponent<'a> {
    pub selected_tab: Tab,
    key_config: &'a KeyConfig,
}

impl<'a> TabComponent<'a> {
    pub fn new(key_config: &'a KeyConfig) -> Self {
        Self {
            selected_tab: Tab::Records,
            key_config,
        }
    }

    pub fn reset(&mut self) {
        self.selected_tab = Tab::Records;
    }

    fn names(&self) -> Vec<String> {
        vec![
            help_info::tab_records(&self.key_config).name,
            help_info::tab_properties(&self.key_config).name,
            help_info::tab_sql_editor(&self.key_config).name,
        ]
    }
}

impl<'a> DrawableComponent for TabComponent<'a> {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, _focused: bool) -> Result<()> {
        let titles = self.names().iter().cloned().map(Spans::from).collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .select(self.selected_tab as usize)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::Reset)
                    .add_modifier(Modifier::UNDERLINED),
            );
        f.render_widget(tabs, area);
        Ok(())
    }
}

impl<'a> Component for TabComponent<'a> {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: &[Key]) -> Result<EventState> {
        if key == [self.key_config.tab_records] {
            self.selected_tab = Tab::Records;
            return Ok(EventState::Consumed);
        } else if key == [self.key_config.tab_sql_editor] {
            self.selected_tab = Tab::Sql;
            return Ok(EventState::Consumed);
        } else if key == [self.key_config.tab_properties] {
            self.selected_tab = Tab::Properties;
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}
