use super::{Component, EventState, StatefulDrawableComponent};
use crate::components::help_info::HelpInfo;
use crate::config::{Connection, KeyConfig, Settings};
use crate::event::Key;
use crate::clipboard::copy_to_clipboard;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub struct ConnectionsComponent<'a> {
    connections: &'a Vec<Connection>,
    state: ListState,
    key_config: &'a KeyConfig,
    settings: &'a Settings,
}

impl<'a> ConnectionsComponent<'a> {
    pub fn new(key_config: &'a KeyConfig, connections: &'a Vec<Connection>, settings: &'a Settings) -> Self {
        let mut state = ListState::default();
        if !connections.is_empty() {
            state.select(Some(0));
        }
        Self {
            connections,
            key_config,
            state,
            settings,
        }
    }

    fn next_connection(&mut self, lines: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i + lines >= self.connections.len() {
                    Some(self.connections.len() - 1)
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
        if self.connections.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    fn scroll_to_bottom(&mut self) {
        if self.connections.is_empty() {
            return;
        }
        self.state.select(Some(self.connections.len() - 1));
    }

    pub fn selected_connection(&self) -> Option<&Connection> {
        match self.state.selected() {
            Some(i) => self.connections.get(i),
            None => None,
        }
    }
}

impl<'a> StatefulDrawableComponent for ConnectionsComponent<'a> {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, _area: Rect, _focused: bool) -> Result<()> {
        let width = 80;
        let height = 20;
        let conns = &self.connections;
        let mut connections: Vec<ListItem> = Vec::new();
        for c in conns.iter() {
            connections.push(
                ListItem::new(vec![Spans::from(Span::raw(c.database_url_with_name()?))])
                    .style(Style::default()),
            )
        }
        let connections = List::new(connections)
            .block(Block::default().borders(Borders::ALL).title("Connections"))
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

impl<'a> Component for ConnectionsComponent<'a> {
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
        } else if key == self.key_config.copy {
            if let Some(c) = self.selected_connection() {
                let s = c.database_url_with_name()?;
                copy_to_clipboard(&s)?;
            }
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}
