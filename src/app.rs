use crate::clipboard::copy_to_clipboard;
use crate::components::{
    CommandInfo, Component as _, DrawableComponent as _, EventState, StatefulDrawableComponent,
};
use crate::database::{MySqlPool, Pool, PostgresPool, SqlitePool, MssqlPool, RECORDS_LIMIT_PER_PAGE};
use crate::event::{Key, Event, Store};
use crate::{
    components::tab::Tab,
    components::{
        command, ConnectionsComponent, DatabasesComponent, ErrorComponent, HelpComponent,
        PropertiesComponent, RecordTableComponent, SqlEditorComponent, TabComponent,
    },
    config::{Config, Connection},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc;

pub enum Focus {
    DabataseList,
    Table,
    ConnectionList,
}
pub struct App<'a> {
    record_table: RecordTableComponent,
    properties: PropertiesComponent,
    sql_editor: SqlEditorComponent,
    focus: Focus,
    tab: TabComponent,
    help: HelpComponent,
    databases: DatabasesComponent,
    connections: ConnectionsComponent<'a>,
    pool: Option<Box<dyn Pool>>,
    left_main_chunk_percentage: u16,
    pub config: Config,
    pub error: ErrorComponent<'a>,
    pub store: Store,
}

impl<'a> App<'a> {
    pub fn new(config: &'a Config, sender: mpsc::Sender<Event>) -> App<'a> {
        let store = Store::new(sender);
        Self {
            config: config.clone(),
            connections: ConnectionsComponent::new(&config.key_config, &config.conn, &config.theme_config),
            record_table: RecordTableComponent::new(config.key_config.clone(), config.theme_config.clone()),
            properties: PropertiesComponent::new(config.key_config.clone(), config.theme_config.clone()),
            sql_editor: SqlEditorComponent::new(config.key_config.clone(), config.theme_config.clone()),
            tab: TabComponent::new(config.key_config.clone()),
            help: HelpComponent::new(config.key_config.clone()),
            databases: DatabasesComponent::new(config.key_config.clone(), config.theme_config.clone()),
            error: ErrorComponent::new(&config.key_config),
            focus: Focus::ConnectionList,
            pool: None,
            left_main_chunk_percentage: 15,
            store,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let Focus::ConnectionList = self.focus {
            self.connections.draw(
                f,
                Layout::default()
                    .constraints([Constraint::Percentage(100)])
                    .split(f.size())[0],
                false,
            )?;
            self.error.draw(f, Rect::default(), false)?;
            self.help.draw(f, Rect::default(), false)?;
            return Ok(());
        }

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.left_main_chunk_percentage),
                Constraint::Percentage((100_u16).saturating_sub(self.left_main_chunk_percentage)),
            ])
            .split(f.size());

        self.databases
            .draw(f, main_chunks[0], matches!(self.focus, Focus::DabataseList))?;

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(5)].as_ref())
            .split(main_chunks[1]);

        self.tab.draw(f, right_chunks[0], false)?;

        match self.tab.selected_tab {
            Tab::Records => {
                self.record_table
                    .draw(f, right_chunks[1], matches!(self.focus, Focus::Table))?
            }
            Tab::Sql => {
                self.sql_editor
                    .draw(f, right_chunks[1], matches!(self.focus, Focus::Table))?;
            }
            Tab::Properties => {
                self.properties
                    .draw(f, right_chunks[1], matches!(self.focus, Focus::Table))?;
            }
        }
        self.error.draw(f, Rect::default(), false)?;
        self.help.draw(f, Rect::default(), false)?;
        Ok(())
    }

    fn update_commands(&mut self) {
        self.help.set_cmds(self.commands());
    }

    fn commands(&self) -> Vec<CommandInfo> {
        let mut res = vec![
            CommandInfo::new(command::filter(&self.config.key_config)),
            CommandInfo::new(command::help(&self.config.key_config)),
            CommandInfo::new(command::toggle_tabs(&self.config.key_config)),
            CommandInfo::new(command::scroll(&self.config.key_config)),
            CommandInfo::new(command::scroll_to_top_bottom(&self.config.key_config)),
            CommandInfo::new(command::scroll_up_down_multiple_lines(
                &self.config.key_config,
            )),
            CommandInfo::new(command::move_focus(&self.config.key_config)),
            CommandInfo::new(command::extend_or_shorten_widget_width(
                &self.config.key_config,
            )),
        ];

        self.databases.commands(&mut res);
        self.record_table.commands(&mut res);
        self.properties.commands(&mut res);

        res
    }

    pub async fn update_databases_internal(&mut self, conn: Option<&Connection>) -> anyhow::Result<()> {
        if conn.is_none() {
            return Ok(())
        }
        let conn = conn.unwrap();
        if let Some(pool) = self.pool.as_ref() {
            pool.close().await;
        }
        self.pool = if conn.is_mysql() {
            Some(Box::new(
                MySqlPool::new(conn.database_url()?.as_str()).await?,
            ))
        } else if conn.is_postgres() {
            Some(Box::new(
                PostgresPool::new(conn.database_url()?.as_str()).await?,
            ))
        } else {
            Some(Box::new(
                SqlitePool::new(conn.database_url()?.as_str()).await?,
            ))
        };
        self.databases
            .update(conn, self.pool.as_ref().unwrap())
            .await?;
        self.focus = Focus::DabataseList;
        self.record_table.reset();
        self.tab.reset();
        Ok(())
    }

    async fn update_databases(&mut self, is_focus: bool) -> anyhow::Result<()> {
        if let Some(conn) = self.connections.selected_connection() {
            if let Some(pool) = self.pool.as_ref() {
                pool.close().await;
            }
            self.pool = if conn.is_mysql() {
                Some(Box::new(
                    MySqlPool::new(conn.database_url()?.as_str()).await?,
                ))
            } else if conn.is_postgres() {
                Some(Box::new(
                    PostgresPool::new(conn.database_url()?.as_str()).await?,
                ))
            } else if conn.is_mssql() {
                Some(Box::new(
                    MssqlPool::new(conn.database_url()?.as_str()).await?,
                ))
            } else {
                Some(Box::new(
                    SqlitePool::new(conn.database_url()?.as_str()).await?,
                ))
            };
            self.databases
                .update(conn, self.pool.as_ref().unwrap())
                .await?;
            if is_focus { self.focus = Focus::DabataseList; }
            self.record_table.reset();
            self.tab.reset();
        }
        Ok(())
    }

    async fn update_record_table(&mut self, focus: bool) -> anyhow::Result<()> {
        if let Some((database, table, _)) = self.databases.tree().selected_table() {
            let (headers, records) = self
                .pool
                .as_ref()
                .unwrap()
                .get_records(
                    &database,
                    &table,
                    0,
                    if self.record_table.filter.input_str().is_empty() {
                        None
                    } else {
                        Some(self.record_table.filter.input_str())
                    },
                )
                .await?;
            self.record_table
                .update(records, headers, database.clone(), table.clone());
            if focus { self.record_table.focus = crate::components::record_table::Focus::Table; }
        }
        Ok(())
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        self.update_commands();

        if self.components_event(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        if self.move_focus(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        };
        Ok(EventState::NotConsumed)
    }

    pub async fn action_event(&mut self, r: Event) -> anyhow::Result<EventState> {
        match r {
            Event::RedrawDatabase(focus) => {
                self.update_databases(focus).await?;
                return Ok(EventState::Consumed)
            },
            Event::RedrawTable(focus) => {
                self.update_record_table(focus).await?;
                return Ok(EventState::Consumed)
            }
            _ => {},
        };
        return Ok(EventState::NotConsumed)
    }

    async fn components_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if self.error.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        if !matches!(self.focus, Focus::ConnectionList) && self.help.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        match self.focus {
            Focus::ConnectionList => {
                if self.connections.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.enter {
                    self.update_databases(true).await?;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::DabataseList => {
                if self.databases.event(key)?.is_consumed() ||
                    self.databases.async_event(key, self.pool.as_ref().unwrap(), &self.store).await?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.enter && self.databases.tree_focused() {
                    if let Some((database, table, _)) = self.databases.tree().selected_table() {
                        self.record_table.reset();
                        let (headers, records) = self
                            .pool
                            .as_ref()
                            .unwrap()
                            .get_records(&database, &table, 0, None)
                            .await?;
                        self.record_table
                            .update(records, headers, database.clone(), table.clone());
                        self.properties
                            .update(database.clone(), table.clone(), self.pool.as_ref().unwrap())
                            .await?;
                        self.focus = Focus::Table;
                    }
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Table => {
                match self.tab.selected_tab {
                    Tab::Records => {
                        if self.record_table.event(key)?.is_consumed() || 
                        self.record_table.async_event(key, self.pool.as_ref().unwrap(), &self.store).await?.is_consumed() {
                            return Ok(EventState::Consumed);
                        };

                        if key == self.config.key_config.copy {
                            if let Some(text) = self.record_table.table.selected_cells() {
                                copy_to_clipboard(text.as_str())?
                            }
                        }

                        if key == self.config.key_config.enter && self.record_table.filter_focused()
                        {
                            self.update_record_table(true).await?;
                        }

                        if self.record_table.table.eod {
                            return Ok(EventState::Consumed);
                        }

                        if let Some(index) = self.record_table.table.selected_row.selected() {
                            if index.saturating_add(1) % RECORDS_LIMIT_PER_PAGE as usize == 0 {
                                if let Some((database, table, _)) =
                                    self.databases.tree().selected_table()
                                {
                                    let (_, records) = self
                                        .pool
                                        .as_ref()
                                        .unwrap()
                                        .get_records(
                                            &database,
                                            &table,
                                            index as u16,
                                            if self.record_table.filter.input_str().is_empty() {
                                                None
                                            } else {
                                                Some(self.record_table.filter.input_str())
                                            },
                                        )
                                        .await?;
                                    if !records.is_empty() {
                                        self.record_table.table.rows.extend(records);
                                    } else {
                                        self.record_table.table.end()
                                    }
                                }
                            }
                        };
                    }
                    Tab::Sql => {
                        if self.sql_editor.event(key)?.is_consumed()
                            || self
                                .sql_editor
                                .async_event(key, self.pool.as_ref().unwrap(), &self.store)
                                .await?
                                .is_consumed()
                        {
                            return Ok(EventState::Consumed);
                        };
                    }
                    Tab::Properties => {
                        if self.properties.event(key)?.is_consumed() {
                            return Ok(EventState::Consumed);
                        };
                    }
                };
            }
        }

        if self.extend_or_shorten_widget_width(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        Ok(EventState::NotConsumed)
    }

    fn extend_or_shorten_widget_width(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key
            == self
                .config
                .key_config
                .extend_or_shorten_widget_width_to_left
        {
            self.left_main_chunk_percentage =
                self.left_main_chunk_percentage.saturating_sub(5).max(15);
            return Ok(EventState::Consumed);
        } else if key
            == self
                .config
                .key_config
                .extend_or_shorten_widget_width_to_right
        {
            self.left_main_chunk_percentage = (self.left_main_chunk_percentage + 5).min(70);
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }

    fn move_focus(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.config.key_config.focus_connections {
            self.focus = Focus::ConnectionList;
            return Ok(EventState::Consumed);
        }
        if self.tab.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }
        match self.focus {
            Focus::ConnectionList => {
                if key == self.config.key_config.enter {
                    self.focus = Focus::DabataseList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::DabataseList => {
                if key == self.config.key_config.focus_right && self.databases.tree_focused() {
                    self.focus = Focus::Table;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Table => {
                if key == self.config.key_config.focus_left {
                    self.focus = Focus::DabataseList;
                    return Ok(EventState::Consumed);
                }
            }
        }
        Ok(EventState::NotConsumed)
    }
}

#[cfg(test)]
mod test {
    use super::{App, Config, EventState, Key};
    use crate::event::Events;

    #[test]
    fn test_extend_or_shorten_widget_width() {
        let events = Events::new(250);
        let c = Config::default();
        let mut app = App::new(&c, events.sender());
        assert_eq!(
            app.extend_or_shorten_widget_width(Key::Char('>')).unwrap(),
            EventState::Consumed
        );
        assert_eq!(app.left_main_chunk_percentage, 20);

        app.left_main_chunk_percentage = 70;
        assert_eq!(
            app.extend_or_shorten_widget_width(Key::Char('>')).unwrap(),
            EventState::Consumed
        );
        assert_eq!(app.left_main_chunk_percentage, 70);

        assert_eq!(
            app.extend_or_shorten_widget_width(Key::Char('<')).unwrap(),
            EventState::Consumed
        );
        assert_eq!(app.left_main_chunk_percentage, 65);

        app.left_main_chunk_percentage = 15;
        assert_eq!(
            app.extend_or_shorten_widget_width(Key::Char('<')).unwrap(),
            EventState::Consumed
        );
        assert_eq!(app.left_main_chunk_percentage, 15);
    }
}
