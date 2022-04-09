use super::{
    utils::scroll_vertical::VerticalScroll, Component, DatabaseFilterComponent, DrawableComponent,
    EventState,
};
use crate::components::help_info::{self, HelpInfo};
use crate::config::{Connection, KeyConfig, Settings};
use crate::database::{Pool};
use crate::event::{Key, Store};
use crate::clipboard::copy_to_clipboard;
use crate::ui::common_nav;
use crate::ui::scrolllist::draw_list_block;
use anyhow::Result;
use database_tree::{Database, DatabaseTree, DatabaseTreeItem};
use async_trait::async_trait;
use std::collections::BTreeSet;
use std::convert::From;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders},
    Frame,
};

// ▸
const FOLDER_ICON_COLLAPSED: &str = "\u{25b8}";
// ▾
const FOLDER_ICON_EXPANDED: &str = "\u{25be}";
const EMPTY_STR: &str = "";

#[derive(PartialEq)]
pub enum Focus {
    Filter,
    Tree,
}

pub struct DatabasesComponent<'a> {
    tree: DatabaseTree,
    filter: DatabaseFilterComponent,
    filterd_tree: Option<DatabaseTree>,
    scroll: VerticalScroll,
    focus: Focus,
    key_config: &'a KeyConfig,
    settings: &'a Settings,
}

// impl Default for DatabasesComponent {
//     fn default() -> Self {
//         Self::new(KeyConfig::default(), ThemeConfig::default())
//     }
// }

impl<'a> DatabasesComponent<'a> {
    pub fn new(key_config: &'a KeyConfig, settings: &'a Settings) -> Self {
        Self {
            tree: DatabaseTree::default(),
            filter: DatabaseFilterComponent::new(),
            filterd_tree: None,
            scroll: VerticalScroll::new(false, false),
            focus: Focus::Tree,
            key_config,
            settings,
        }
    }

    pub async fn update(&mut self, connection: &Connection, pool: &Box<dyn Pool>) -> Result<()> {
        // TODO: load schema first
        let databases = match &connection.database {
            Some(database) => vec![Database::new(
                database.clone(),
                pool.get_tables(database.clone()).await?,
            )],
            None => pool.get_databases().await?,
        };
        self.tree = DatabaseTree::new(databases.as_slice(), &BTreeSet::new())?;
        self.filterd_tree = None;
        self.filter.reset();
        Ok(())
    }

    pub fn tree_focused(&self) -> bool {
        matches!(self.focus, Focus::Tree)
    }

    pub fn tree(&self) -> &DatabaseTree {
        self.filterd_tree.as_ref().unwrap_or(&self.tree)
    }

    fn tree_item_to_span(
        &self,
        item: DatabaseTreeItem,
        selected: bool,
        width: u16,
        filter: Option<String>,
    ) -> Spans<'static> {
        let name = item.kind().name();
        let indent = item.info().indent();

        let indent_str = if indent == 0 {
            String::from("")
        } else {
            format!("{:w$}", " ", w = (indent as usize) * 2)
        };

        let arrow = if item.kind().is_database() || item.kind().is_schema() {
            if item.kind().is_database_collapsed() || item.kind().is_schema_collapsed() {
                FOLDER_ICON_COLLAPSED
            } else {
                FOLDER_ICON_EXPANDED
            }
        } else {
            EMPTY_STR
        };

        if let Some(filter) = filter {
            if item.kind().is_table() && name.contains(&filter) {
                let (first, rest) = &name.split_at(name.find(filter.as_str()).unwrap_or(0));
                let (middle, last) = &rest.split_at(filter.len().clamp(0, rest.len()));
                return Spans::from(vec![
                    Span::styled(
                        format!("{}{}{}", indent_str, arrow, first),
                        if selected {
                            Style::default().bg(self.settings.color)
                        } else {
                            Style::default()
                        },
                    ),
                    Span::styled(
                        middle.to_string(),
                        if selected {
                            Style::default().bg(self.settings.color).fg(self.settings.color)
                        } else {
                            Style::default().fg(self.settings.color)
                        },
                    ),
                    Span::styled(
                        format!("{:w$}", last.to_string(), w = width as usize),
                        if selected {
                            Style::default().bg(self.settings.color)
                        } else {
                            Style::default()
                        },
                    ),
                ]);
            }
        }

        Spans::from(Span::styled(
            format!("{}{}{:w$}", indent_str, arrow, name, w = width as usize),
            if selected {
                Style::default().bg(self.settings.color)
            } else {
                Style::default()
            },
        ))
    }

    fn draw_tree<B: Backend>(&self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        f.render_widget(
            Block::default()
                .title("Databases")
                .borders(Borders::ALL)
                .style(if focused {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
            area,
        );

        let chunks = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
            .split(area);

        self.filter
            .draw(f, chunks[0], matches!(self.focus, Focus::Filter))?;

        let tree_height = chunks[1].height as usize;
        let tree = if let Some(tree) = self.filterd_tree.as_ref() {
            tree
        } else {
            &self.tree
        };
        tree.visual_selection().map_or_else(
            || {
                self.scroll.reset();
            },
            |selection| {
                self.scroll
                    .update(selection.index, selection.count, tree_height);
            },
        );

        let items = tree
            .iterate(self.scroll.get_top(), tree_height)
            .map(|(item, selected)| {
                self.tree_item_to_span(
                    item.clone(),
                    selected,
                    area.width,
                    if self.filter.input_str().is_empty() {
                        None
                    } else {
                        Some(self.filter.input_str())
                    },
                )
            });

        draw_list_block(f, chunks[1], Block::default().borders(Borders::NONE), items);
        self.scroll.draw(f, chunks[1]);

        Ok(())
    }
}

impl<'a> DrawableComponent for DatabasesComponent<'a> {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area);

        self.draw_tree(f, chunks[0], focused)?;
        Ok(())
    }
}

#[async_trait]
impl<'a> Component for DatabasesComponent<'a> {
    fn helps(&self, out: &mut Vec<HelpInfo>) {
        out.push(HelpInfo::new(help_info::expand_collapse(&self.key_config)))
    }

    fn event(&mut self, key: &[Key]) -> Result<EventState> {
        if key[0] == self.key_config.filter && self.focus == Focus::Tree {
            self.focus = Focus::Filter;
            return Ok(EventState::Consumed);
        }

        if matches!(self.focus, Focus::Filter) {
            self.filterd_tree = if self.filter.input_str().is_empty() {
                None
            } else {
                Some(self.tree.filter(self.filter.input_str()))
            };
        }

        match key {
            [Key::Enter] if matches!(self.focus, Focus::Filter) => {
                self.focus = Focus::Tree;
                return Ok(EventState::Consumed);
            }
            key if matches!(self.focus, Focus::Filter) => {
                if self.filter.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            key if key[0] == self.key_config.copy => {
                if let Some(item) = self.tree.selected_item() {
                    let name = item.kind().name();
                    copy_to_clipboard(&name)?;
                }
                return Ok(EventState::Consumed);
            }
            key => {
                if tree_nav(
                    if let Some(tree) = self.filterd_tree.as_mut() {
                        tree
                    } else {
                        &mut self.tree
                    },
                    key[0],
                    &self.key_config,
                ) {
                    return Ok(EventState::Consumed);
                }
            }
        }
        Ok(EventState::NotConsumed)
    }

    async fn async_event(
        &mut self,
        key: crate::event::Key,
        pool: &Box<dyn Pool>,
        _store: &Store,
    ) -> Result<EventState> {
        if key == self.key_config.delete {
            if let Some((database, table, id)) = self.tree.selected_table() {
                let sql = pool.database_type().drop_table(&database, &table);
                pool.execute(&sql).await?;
                self.tree = self.tree.filter_by_id(id, true);
            }
            return Ok(EventState::Consumed)
        }
        // self.key_config.advanced_copy
        if key == self.key_config.advanced_copy {
            if let Some((database, table, _)) = self.tree.selected_table() {
                let ddl = pool.database_type().show_schema(pool, &database, &table).await?;
                copy_to_clipboard(&ddl)?;
            }
        }
        Ok(EventState::NotConsumed)
    }
}

fn tree_nav(tree: &mut DatabaseTree, key: Key, key_config: &KeyConfig) -> bool {
    if let Some(common_nav) = common_nav(key, key_config) {
        tree.move_selection(common_nav)
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use super::{Color, Database, DatabaseTreeItem, DatabasesComponent, Span, Spans, Style, KeyConfig, Settings};
    use database_tree::Table;

    #[test]
    fn test_tree_database_tree_item_to_span() {
        const WIDTH: u16 = 10;
        let key = KeyConfig::default(); 
        let settings = Settings::default();
        let dc = DatabasesComponent::new(&key, &settings);
        assert_eq!(
            dc.tree_item_to_span(
                DatabaseTreeItem::new_database(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    false,
                ),
                false,
                WIDTH,
                None,
            ),
            Spans::from(vec![Span::raw(format!(
                "\u{25b8}{:w$}",
                "foo",
                w = WIDTH as usize
            ))])
        );

        assert_eq!(
            dc.tree_item_to_span(
                DatabaseTreeItem::new_database(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    false,
                ),
                true,
                WIDTH,
                None,
            ),
            Spans::from(vec![Span::styled(
                format!("\u{25b8}{:w$}", "foo", w = WIDTH as usize),
                Style::default().bg(Color::Blue)
            )])
        );
    }

    #[test]
    fn test_tree_table_tree_item_to_span() {
        const WIDTH: u16 = 10;
        let key = KeyConfig::default(); 
        let settings = Settings::default();
        let dc = DatabasesComponent::new(&key, &settings);
        assert_eq!(
            dc.tree_item_to_span(
                DatabaseTreeItem::new_table(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    &Table {
                        name: "bar".to_string(),
                        create_time: None,
                        update_time: None,
                        engine: None,
                        schema: None
                    },
                ),
                false,
                WIDTH,
                None,
            ),
            Spans::from(vec![Span::raw(format!(
                "  {:w$}",
                "bar",
                w = WIDTH as usize
            ))])
        );

        assert_eq!(
            dc.tree_item_to_span(
                DatabaseTreeItem::new_table(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    &Table {
                        name: "bar".to_string(),
                        create_time: None,
                        update_time: None,
                        engine: None,
                        schema: None
                    },
                ),
                true,
                WIDTH,
                None,
            ),
            Spans::from(Span::styled(
                format!("  {:w$}", "bar", w = WIDTH as usize),
                Style::default().bg(Color::Blue),
            ))
        );
    }

    #[test]
    fn test_filterd_tree_item_to_span() {
        const WIDTH: u16 = 10;
        let key = KeyConfig::default(); 
        let settings = Settings::default();
        let dc = DatabasesComponent::new(&key, &settings);
        assert_eq!(
            dc.tree_item_to_span(
                DatabaseTreeItem::new_table(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    &Table {
                        name: "barbaz".to_string(),
                        create_time: None,
                        update_time: None,
                        engine: None,
                        schema: None
                    },
                ),
                false,
                WIDTH,
                Some("rb".to_string()),
            ),
            Spans::from(vec![
                Span::raw(format!("  {}", "ba")),
                Span::styled("rb", Style::default().fg(Color::Blue)),
                Span::raw(format!("{:w$}", "az", w = WIDTH as usize))
            ])
        );

        assert_eq!(
            dc.tree_item_to_span(
                DatabaseTreeItem::new_table(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    &Table {
                        name: "barbaz".to_string(),
                        create_time: None,
                        update_time: None,
                        engine: None,
                        schema: None
                    },
                ),
                true,
                WIDTH,
                Some("rb".to_string()),
            ),
            Spans::from(vec![
                Span::styled(format!("  {}", "ba"), Style::default().bg(Color::Blue)),
                Span::styled("rb", Style::default().bg(Color::Blue).fg(Color::Blue)),
                Span::styled(
                    format!("{:w$}", "az", w = WIDTH as usize),
                    Style::default().bg(Color::Blue)
                )
            ])
        );
    }
}
