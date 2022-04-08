use super::{Component, EventState, StatefulDrawableComponent};
use crate::components::command::CommandInfo;
use crate::components::{TableComponent, TableFilterComponent};
use crate::config::{KeyConfig, Settings};
use crate::event::Key;
use crate::database::{Pool, Header, Value};
use anyhow::Result;
use database_tree::{Database, Table as DTable};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use async_trait::async_trait;

pub enum Focus {
    Table,
    Filter,
}

pub struct RecordTableComponent {
    pub filter: TableFilterComponent,
    pub table: TableComponent,
    pub focus: Focus,
    key_config: KeyConfig,
}

impl RecordTableComponent {
    pub fn new(key_config: KeyConfig, theme: Settings) -> Self {
        Self {
            filter: TableFilterComponent::new(key_config.clone(), theme.clone()),
            table: TableComponent::new(key_config.clone(), theme),
            focus: Focus::Table,
            key_config,
        }
    }

    pub fn update(
        &mut self,
        rows: Vec<Vec<Value>>,
        headers: Vec<Header>,
        database: Database,
        table: DTable,
        selected_column: usize,
    ) {
        let candidates = headers.iter().map(|h| h.name.clone()).collect::<Vec<_>>();
        self.table.update(rows, headers, database, table.clone(), selected_column);
        self.filter.table = Some(table);
        self.filter.update_candidates(&candidates);
    }

    pub fn reset(&mut self) {
        self.table.reset();
        self.filter.reset();
    }

    pub fn filter_focused(&self) -> bool {
        matches!(self.focus, Focus::Filter)
    }
}

impl StatefulDrawableComponent for RecordTableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Length(5)])
            .split(area);

        self.table
            .draw(f, layout[1], focused && matches!(self.focus, Focus::Table))?;

        self.filter
            .draw(f, layout[0], focused && matches!(self.focus, Focus::Filter))?;
        Ok(())
    }
}

#[async_trait]
impl Component for RecordTableComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        self.table.commands(out)
    }

    fn event(&mut self, key: &[Key]) -> Result<EventState> {
        if key[0] == self.key_config.filter {
            self.focus = Focus::Filter;
            return Ok(EventState::Consumed);
        }
        match key {
            key if matches!(self.focus, Focus::Filter) => return self.filter.event(key),
            key if matches!(self.focus, Focus::Table) => return self.table.event(key),
            _ => (),
        }
        Ok(EventState::NotConsumed)
    }

    async fn async_event(
        &mut self,
        key: crate::event::Key,
        pool: &Box<dyn Pool>,
        store: &crate::event::Store,
    ) -> Result<EventState> {
        match key {
            key if matches!(self.focus, Focus::Filter) => return self.filter.async_event(key, pool, store).await,
            key if matches!(self.focus, Focus::Table) => return self.table.async_event(key, pool, store).await,
            _ => (),
        }
        Ok(EventState::NotConsumed)
    }    

}
