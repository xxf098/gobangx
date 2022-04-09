use super::{Component, EventState, StatefulDrawableComponent};
use crate::components::help_info::HelpInfo;
use crate::config::{KeyConfig, Settings};
use crate::event::Key;
use database_tree::{Database, Table};
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub struct Recent {
    database: Database,
    table: Table,
}

pub struct RecentComponent<'a> {
    recents: &'a Vec<Recent>,
    state: ListState,
    key_config: &'a KeyConfig,
    settings: &'a Settings,
}

impl<'a> RecentComponent<'a> {
    pub fn new(key_config: &'a KeyConfig, recents: &'a Vec<Recent>, settings: &'a Settings) -> Self {
        let mut state = ListState::default();
        if !recents.is_empty() {
            state.select(Some(0));
        }
        Self {
            recents,
            key_config,
            state,
            settings,
        }
    }

    fn next_connection(&mut self, lines: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i + lines >= self.recents.len() {
                    Some(self.recents.len() - 1)
                } else {
                    Some(i + lines)
                }
            }
            None => None,
        };
        self.state.select(i);
    }

    fn previous_connection(&mut self, lines: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= lines {
                    Some(0)
                } else {
                    Some(i - lines)
                }
            }
            None => None,
        };
        self.state.select(i);
    }

    fn scroll_to_top(&mut self) {
        if self.recents.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    fn scroll_to_bottom(&mut self) {
        if self.recents.is_empty() {
            return;
        }
        self.state.select(Some(self.recents.len() - 1));
    }

}


impl<'a> StatefulDrawableComponent for RecentComponent<'a> {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, _area: Rect, _focused: bool) -> Result<()> {
        let width = 80;
        let height = 20;
        let conns = &self.recents;
        let mut connections: Vec<ListItem> = Vec::new();
        for c in conns.iter() {
            connections.push(
                ListItem::new(vec![Spans::from(Span::raw(c.table.name.clone()))])
                    .style(Style::default()),
            )
        }
        let connections = List::new(connections)
            .block(Block::default().borders(Borders::ALL).title("Recents"))
            .highlight_style(Style::default().bg(self.settings.color))
            .style(Style::default());

        let area = Rect::new(
            (f.size().width.saturating_sub(width)) / 2,
            (f.size().height.saturating_sub(height)) / 2,
            width.min(f.size().width),
            height.min(f.size().height),
        );

        f.render_widget(Clear, area);
        f.render_stateful_widget(connections, area, &mut self.state);
        Ok(())
    }
}

impl<'a> Component for RecentComponent<'a> {
    fn commands(&self, _out: &mut Vec<HelpInfo>) {}

    fn event(&mut self, key: &[Key]) -> Result<EventState> {
        let key = key[0];
        if key == self.key_config.scroll_down {
            self.next_connection(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up {
            self.previous_connection(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next_connection(10);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous_connection(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_top {
            self.scroll_to_top();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_bottom {
            self.scroll_to_bottom();
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}