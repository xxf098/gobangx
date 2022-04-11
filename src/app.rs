use crate::clipboard::copy_to_clipboard;
use crate::components::{
    HelpInfo, Component as _, DrawableComponent as _, EventState, StatefulDrawableComponent,
};
use crate::database::{MySqlPool, Pool, PostgresPool, SqlitePool, MssqlPool};
use crate::event::{Key, Event, Store};
use crate::config::DatabaseType;
use crate::{
    components::tab::Tab,
    components::{
        help_info, ConnectionsComponent, DatabasesComponent, ErrorComponent, HelpComponent,
        PropertiesComponent, RecordTableComponent, SqlEditorComponent, TabComponent, RecentComponent, Recent
    },
    config::{Config, Connection},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::collections::VecDeque;

pub enum Focus {
    DabataseList,
    Table,
    ConnectionList,
    RecentList,
}
pub struct App<'a> {
    record_table: RecordTableComponent,
    properties: PropertiesComponent<'a>,
    sql_editor: SqlEditorComponent<'a>,
    focus: Focus,
    tab: TabComponent<'a>,
    help: HelpComponent<'a>,
    databases: DatabasesComponent<'a>,
    show_database: bool,
    connections: ConnectionsComponent<'a>,
    recents: RecentComponent<'a>,
    pool: Option<Box<dyn Pool>>,
    left_main_chunk_percentage: u16,
    pub config: Config,
    pub error: ErrorComponent<'a>,
    pub store: Store,
    pub keys: Vec<Key>,
}

impl<'a> App<'a> {
    pub fn new(config: &'a Config, sender: mpsc::Sender<Event>) -> App<'a> {
        let store = Store::new(sender);
        let mut app = Self {
            config: config.clone(),
            connections: ConnectionsComponent::new(&config.key_config, &config.conn, &config.settings),
            record_table: RecordTableComponent::new(config.key_config.clone(), config.settings.clone()),
            properties: PropertiesComponent::new(&config.key_config, &config.settings),
            sql_editor: SqlEditorComponent::new(&config.key_config, config.settings.clone(), DatabaseType::Sqlite),
            tab: TabComponent::new(&config.key_config),
            help: HelpComponent::new(&config.key_config),
            databases: DatabasesComponent::new(&config.key_config, &config.settings),
            recents: RecentComponent::new(&config.key_config, VecDeque::new(), &config.settings),
            show_database: true,
            error: ErrorComponent::new(&config.key_config),
            focus: Focus::ConnectionList,
            pool: None,
            left_main_chunk_percentage: 15,
            store,
            keys: Vec::with_capacity(8),
        };
        app.update_helps();
        app
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

        if let Focus::RecentList = self.focus {
            self.recents.draw(
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

        let left_main_chunk_percentage = if self.show_database { self.left_main_chunk_percentage } else { 0 };
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(left_main_chunk_percentage),
                Constraint::Percentage((100_u16).saturating_sub(left_main_chunk_percentage)),
            ])
            .split(f.size());

        if self.show_database {
            self.databases
              .draw(f, main_chunks[0], matches!(self.focus, Focus::DabataseList))?;
        }

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

    fn update_helps(&mut self) {
        self.help.set_cmds(self.helps());
    }

    fn helps(&self) -> Vec<HelpInfo> {
        let mut res = vec![
            HelpInfo::new(help_info::filter(&self.config.key_config)),
            HelpInfo::new(help_info::help(&self.config.key_config)),
            HelpInfo::new(help_info::toggle_tabs(&self.config.key_config)),
            HelpInfo::new(help_info::scroll(&self.config.key_config)),
            HelpInfo::new(help_info::scroll_to_top_bottom(&self.config.key_config)),
            HelpInfo::new(help_info::scroll_up_down_multiple_lines(
                &self.config.key_config,
            )),
            HelpInfo::new(help_info::move_focus(&self.config.key_config)),
            HelpInfo::new(help_info::extend_or_shorten_widget_width(
                &self.config.key_config,
            )),
        ];

        self.databases.helps(&mut res);
        self.record_table.helps(&mut res);
        self.properties.helps(&mut res);
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
        let page_size = self.config.settings.page_size;
        self.pool = if conn.is_mysql() {
            Some(Box::new(
                MySqlPool::new(conn.database_url()?.as_str(), page_size).await?,
            ))
        } else if conn.is_postgres() {
            Some(Box::new(
                PostgresPool::new(conn.database_url()?.as_str(), page_size).await?,
            ))
        } else {
            Some(Box::new(
                SqlitePool::new(conn.database_url()?.as_str(), page_size).await?,
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
            let page_size = self.config.settings.page_size;
            self.pool = if conn.is_mysql() {
                self.sql_editor.set_database_type(DatabaseType::MySql);
                Some(Box::new(
                    MySqlPool::new(conn.database_url()?.as_str(), page_size).await?,
                ))
            } else if conn.is_postgres() {
                self.sql_editor.set_database_type(DatabaseType::Postgres);
                Some(Box::new(
                    PostgresPool::new(conn.database_url()?.as_str(), page_size).await?,
                ))
            } else if conn.is_mssql() {
                self.sql_editor.set_database_type(DatabaseType::Mssql);
                Some(Box::new(
                    MssqlPool::new(conn.database_url()?.as_str(), page_size).await?,
                ))
            } else {
                self.sql_editor.set_database_type(DatabaseType::Sqlite);
                Some(Box::new(
                    SqlitePool::new(conn.database_url()?.as_str(), page_size).await?,
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

    async fn update_record_table(&mut self, focus: bool, orderby: Option<String>, selected_column: usize) -> anyhow::Result<()> {
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
                    orderby,
                )
                .await?;
            self.record_table
                .update(records, headers, database.clone(), table.clone(), selected_column);
            if focus { self.record_table.focus = crate::components::record_table::Focus::Table; }
        }
        Ok(())
    }

    pub fn clear_keys(&mut self) {
        self.keys.clear()
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        // self.update_commands();
        self.keys.push(key);
        if self.components_event(self.keys.clone()).await?.is_consumed() {
            self.keys.clear();
            return Ok(EventState::Consumed);
        };

        if self.move_focus()?.is_consumed() {
            self.keys.clear();
            return Ok(EventState::Consumed);
        };
        // self.keys.clear();
        Ok(EventState::NotConsumed)
    }

    pub async fn action_event(&mut self, r: Event) -> anyhow::Result<EventState> {
        match r {
            Event::RedrawDatabase(focus) => {
                self.update_databases(focus).await?;
                return Ok(EventState::Consumed)
            },
            Event::RedrawTable(focus) => {
                self.update_record_table(focus, None, 0).await?;
                return Ok(EventState::Consumed)
            },
            Event::OrderByTable((order, selected_column)) => {
                let orderby = if order.len() > 0 { Some(order) } else { None };
                self.update_record_table(true, orderby, selected_column).await?;
                return Ok(EventState::Consumed)
            }
            Event::ToggleTree => {
                self.show_database = !self.show_database;
                return Ok(EventState::Consumed)
            }
            _ => {},
        };
        return Ok(EventState::NotConsumed)
    }

    async fn components_event(&mut self, key: Vec<Key>) -> anyhow::Result<EventState> {
        if self.error.event(&key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        if !matches!(self.focus, Focus::ConnectionList) && self.help.event(&key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        match self.focus {
            Focus::ConnectionList => {
                if self.connections.event(&key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
                
                if key[0] == self.config.key_config.enter {
                    self.update_databases(true).await?;
                    self.recents.reset();
                    return Ok(EventState::Consumed);
                }
            }
            Focus::RecentList => {
                if self.recents.event(&key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
                if key[0] == self.config.key_config.enter {
                    let recent = self.recents.selected_recent().map(|r| r.clone());
                    if let Some(Recent{id, database, table}) = recent {
                        self.databases.set_selection(id);
                        self.recents.add(id, &database, &table);
                        self.record_table.reset();
                        let (headers, records) = self
                            .pool
                            .as_ref()
                            .unwrap()
                            .get_records(&database, &table, 0, None, None)
                            .await?;
                        self.record_table
                            .update(records, headers, database.clone(), table.clone(), 0);
                        self.properties
                            .update(database.clone(), table.clone(), self.pool.as_ref().unwrap())
                            .await?;
                        self.focus = Focus::Table;
                    }
                }

                if key[0] == self.config.key_config.exit_popup {
                    self.focus = Focus::Table;
                }
               
                return Ok(EventState::Consumed);
            }
            Focus::DabataseList => {
                if self.databases.event(&key)?.is_consumed() ||
                    self.databases.async_event(key[0], self.pool.as_ref().unwrap(), &self.store).await?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key[0] == self.config.key_config.enter && self.databases.tree_focused() {
                    if let Some((database, table, id)) = self.databases.tree().selected_table() {
                        self.recents.add(id, &database, &table);
                        self.record_table.reset();
                        let (headers, records) = self
                            .pool
                            .as_ref()
                            .unwrap()
                            .get_records(&database, &table, 0, None, None)
                            .await?;
                        self.record_table
                            .update(records, headers, database.clone(), table.clone(), 0);
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
                        if self.record_table.event(&key)?.is_consumed() || 
                        self.record_table.async_event(key[0], self.pool.as_ref().unwrap(), &self.store).await?.is_consumed() {
                            return Ok(EventState::Consumed);
                        };

                        if key == [self.config.key_config.copy] {
                            if let Some(text) = self.record_table.table.selected_cells() {
                                copy_to_clipboard(text.as_str())?
                            }
                        }

                        if key[0] == self.config.key_config.enter && self.record_table.filter_focused()
                        {
                            self.update_record_table(true, None, 0).await?;
                        }

                        if key[0] == self.config.key_config.exit_popup && self.record_table.filter_focused()
                        {
                            self.record_table.focus = crate::components::record_table::Focus::Table;
                        }

                        if self.record_table.table.eod {
                            return Ok(EventState::Consumed);
                        }

                        if let Some(index) = self.record_table.table.selected_row.selected() {
                            if index.saturating_add(1) % self.config.settings.page_size as usize == 0 {
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
                                            None,
                                        )
                                        .await?;
                                    if !records.is_empty() {
                                        let records = records.into_iter().map(|row| row.into_iter().map(|cell| Arc::new(RwLock::new(cell))).collect::<Vec<_>>()).collect::<Vec<_>>();
                                        self.record_table.table.rows.extend(records);
                                    } else {
                                        self.record_table.table.end()
                                    }
                                }
                            }
                        };
                    }
                    Tab::Sql => {
                        if self.sql_editor.event(&key)?.is_consumed()
                            || self
                                .sql_editor
                                .async_event(key[0], self.pool.as_ref().unwrap(), &self.store)
                                .await?
                                .is_consumed()
                        {
                            return Ok(EventState::Consumed);
                        };
                    }
                    Tab::Properties => {
                        if self.properties.event(&key)?.is_consumed() {
                            return Ok(EventState::Consumed);
                        };
                    }
                };
            }
        }

        if self.extend_or_shorten_widget_width(key[0])?.is_consumed() {
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

    fn move_focus(&mut self) -> anyhow::Result<EventState> {
        let key = &self.keys;
        if key[0] == self.config.key_config.focus_connections {
            self.focus = Focus::ConnectionList;
            return Ok(EventState::Consumed);
        }
        if key[0] == self.config.key_config.focus_recents {
            self.focus = Focus::RecentList;
            return Ok(EventState::Consumed);
        }

        match self.focus {
            Focus::ConnectionList => {
                if key[0] == self.config.key_config.enter {
                    self.focus = Focus::DabataseList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::RecentList => {
                if key[0] == self.config.key_config.enter {
                    self.focus = Focus::DabataseList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::DabataseList => {
                if key[0] == self.config.key_config.focus_right && self.databases.tree_focused() {
                    self.focus = Focus::Table;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Table => {
                if self.tab.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
                if key[0] == self.config.key_config.focus_left {
                    if self.show_database { self.focus = Focus::DabataseList; }
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

    #[test] #[ignore]
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
