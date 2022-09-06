use super::{
    utils::scroll_vertical::VerticalScroll, Component, DrawableComponent, EventState,
    StatefulDrawableComponent, TableStatusComponent, LineEditorComponent, CommandEditorComponent,
};
use crate::components::help_info::{self, HelpInfo};
use crate::config::{KeyConfig, Settings};
use crate::event::{Key, Store, Event};
use crate::database::{Pool, Header, Value};
use crate::clipboard::copy_to_clipboard;
use anyhow::Result;
use database_tree::{Database, Table as DTable};
use std::convert::{From, Into};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use unicode_width::UnicodeWidthStr;
use async_trait::async_trait;
const NULL: &str = "<NULL>";

#[derive(PartialEq)]
pub enum Focus {
    Status,
    Editor,
    Command,
}

#[derive(Copy, Clone)]
pub enum Movement {
    Forward(char),
    Backward(char)
}

pub struct TableComponent {
    pub headers: Vec<Header>,
    pub rows: Vec<Vec<Arc<RwLock<Value>>>>,
    pub eod: bool,
    pub selected_row: TableState,
    pub focus: Focus,
    constraint_adjust: Vec<u16>, // adjust constraints
    table: Option<(Database, DTable)>,
    selected_column: usize,
    selection_area_corner: Option<(usize, usize)>,
    column_page_start: AtomicUsize,
    scroll: VerticalScroll,
    key_config: KeyConfig,
    settings: Settings,
    area_width: u16,
    cell_editor: LineEditorComponent,
    command_editor: CommandEditorComponent,
    orderby_status: Option<String>,
    movement: Option<Movement>
}

impl TableComponent {
    pub fn new(key_config: KeyConfig, settings: Settings) -> Self {
        Self {
            selected_row: TableState::default(),
            cell_editor: LineEditorComponent::new("".to_string()),
            command_editor: CommandEditorComponent::new("".to_string()),
            headers: vec![],
            rows: vec![],
            table: None,
            constraint_adjust: vec![],
            selected_column: 0,
            selection_area_corner: None,
            column_page_start: AtomicUsize::new(0),
            scroll: VerticalScroll::new(false, false),
            eod: false,
            key_config,
            area_width: 0,
            settings,
            focus: Focus::Status,
            orderby_status: None,
            movement: None,
        }
    }

    fn title(&self) -> String {
        self.table.as_ref().map_or(" - ".to_string(), |table| {
            format!("{}.{}", table.0.name, table.1.name)
        })
    }

    pub fn update(
        &mut self,
        rows: Vec<Vec<Value>>,
        headers: Vec<Header>,
        database: Database,
        table: DTable,
        selected_column: usize,
    ) {
        self.selected_row.select(None);
        if !rows.is_empty() {
            self.selected_row.select(Some(0))
        }
        self.headers = headers;
        self.constraint_adjust = vec![0; self.headers.len()];
        self.rows = rows.into_iter().map(|row| row.into_iter().map(|cell| Arc::new(RwLock::new(cell))).collect::<Vec<_>>()).collect::<Vec<_>>();
        self.selected_column = selected_column;
        self.selection_area_corner = None;
        self.column_page_start = AtomicUsize::new(0);
        self.scroll = VerticalScroll::new(false, false);
        self.eod = false;
        self.table = Some((database, table));
    }

    pub fn reset(&mut self) {
        self.selected_row.select(None);
        self.headers = Vec::new();
        self.rows = Vec::new();
        self.selected_column = 0;
        self.selection_area_corner = None;
        self.column_page_start = AtomicUsize::new(0);
        self.scroll = VerticalScroll::new(false, false);
        self.eod = false;
        self.table = None;
        self.constraint_adjust = vec![];
        self.area_width = 0;
        self.focus = Focus::Status;
        self.orderby_status = None;
        self.movement = None;
    }

    fn reset_selection(&mut self) {
        self.selection_area_corner = None;
    }

    pub fn end(&mut self) {
        self.eod = true;
    }

    fn next_row(&mut self, lines: usize) {
        let i = match self.selected_row.selected() {
            Some(i) => {
                if i + lines >= self.rows.len() {
                    Some(self.rows.len().saturating_sub(1))
                } else {
                    Some(i + lines)
                }
            }
            None => None,
        };
        self.reset_selection();
        self.selected_row.select(i);
    }

    fn previous_row(&mut self, lines: usize) {
        let i = match self.selected_row.selected() {
            Some(i) => {
                if i <= lines {
                    Some(0)
                } else {
                    Some(i.saturating_sub(lines))
                }
            }
            None => None,
        };
        self.reset_selection();
        self.selected_row.select(i);
    }

    fn scroll_to_top(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        self.selected_row.select(Some(0));
    }

    fn scroll_to_bottom(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        self.selected_row
            .select(Some(self.rows.len().saturating_sub(1)));
    }

    fn next_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        if self.selected_column >= self.headers.len().saturating_sub(1) {
            return;
        }
        self.selected_column += 1;
    }

    // jump to last column 
    fn last_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        if self.selected_column >= self.headers.len().saturating_sub(1) {
            return;
        }
        self.selected_column = self.headers.len().saturating_sub(1);
    }

    fn forward_by_character(&mut self, c: char) {
        if let Some((i, _)) = self.headers.iter().enumerate().find(|(i, h)| *i > self.selected_column && h.name.starts_with(c)) {
            self.selected_column = i;
            self.movement = Some(Movement::Forward(c));
        }
    }

    fn backward_by_character(&mut self, c: char) {
        if let Some((i, _)) = self.headers.iter().enumerate().rev().find(|(i, h)| *i < self.selected_column && h.name.starts_with(c)) {
            self.selected_column = i;
            self.movement = Some(Movement::Backward(c));
        }
    }

    fn previous_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        if self.selected_column == 0 {
            return;
        }
        self.selected_column -= 1;
    }

    // jump to first column
    fn first_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        if self.selected_column == 0 {
            return;
        }
        self.selected_column = 0;
    }

    fn expand_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let index = self.selected_column_index()+1;
        let adjust = self.constraint_adjust[index];
        if adjust >= self.area_width/3 {
            return
        }
        self.constraint_adjust[index] = adjust + 1;
    }

    fn shorten_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let index = self.selected_column_index()+1;
        let adjust = self.constraint_adjust[index];
        if adjust < 1 {
            return 
        }
        self.constraint_adjust[index] = adjust - 1;
    }

    // fn reset_column(&mut self) {
    //     if self.rows.is_empty() {
    //         return;
    //     }
    //     let index = self.selected_column_index()+1;
    //     self.constraint_adjust[index] = 0;
    // }

    fn expand_selected_area_x(&mut self, positive: bool) {
        if self.selection_area_corner.is_none() {
            self.selection_area_corner = Some((
                self.selected_column,
                self.selected_row.selected().unwrap_or(0),
            ));
        }
        if let Some((x, y)) = self.selection_area_corner {
            self.selection_area_corner = Some((
                if positive {
                    (x + 1).min(self.headers.len().saturating_sub(1))
                } else {
                    x.saturating_sub(1)
                },
                y,
            ));
        }
    }

    fn expand_selected_area_y(&mut self, positive: bool) {
        if self.selection_area_corner.is_none() {
            self.selection_area_corner = Some((
                self.selected_column,
                self.selected_row.selected().unwrap_or(0),
            ));
        }
        if let Some((x, y)) = self.selection_area_corner {
            self.selection_area_corner = Some((
                x,
                if positive {
                    (y + 1).min(self.rows.len().saturating_sub(1))
                } else {
                    y.saturating_sub(1)
                },
            ));
        }
    }

    pub fn selected_cells(&self) -> Option<String> {
        if let Some((x, y)) = self.selection_area_corner {
            let selected_row_index = self.selected_row.selected()?;
            return Some(
                self.rows[y.min(selected_row_index)..y.max(selected_row_index) + 1]
                    .iter()
                    .map(|row| {
                        row[x.min(self.selected_column)..x.max(self.selected_column) + 1].iter().map(|r| r.read().unwrap().data.clone()).collect::<Vec<String>>().join(",")
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
        }
        self.selected_cell().map(|c| c.to_string())
    }

    pub fn selected_cell(&self) -> Option<Value> {
        self.rows
            .get(self.selected_row.selected()?)?
            .get(self.selected_column)
            .map(|cell| cell.read().unwrap().clone())
    }

    // TODO:
    pub fn set_selected_cell(&mut self, v: String) -> Option<()> {
        let row = self.rows.get_mut(self.selected_row.selected()?)?;
        let value = if v == NULL { Value::default() } else { Value::new(v) };
        let cell = &row[self.selected_column];
        let mut v = cell.write().unwrap();
        *v = value;
        Some(())
    }

    pub fn selected_rows(&self) -> Option<Vec<Vec<Arc<RwLock<Value>>>>> {
        if let Some((_x, y)) = self.selection_area_corner {
            let selected_row_index = self.selected_row.selected()?;
            return Some(
                self.rows[y.min(selected_row_index)..y.max(selected_row_index) + 1]
                    .iter()
                    .map(|row| row.clone())
                    .collect()
            );
        }
        let rows = self.rows.get(self.selected_row.selected()?)?;
        Some(vec![rows.to_vec()])
    }

    fn selected_column_index(&self) -> usize {
        if let Some((x, _)) = self.selection_area_corner {
            return x;
        }
        self.selected_column
    }

    fn is_selected_cell(
        &self,
        row_index: usize,
        column_index: usize,
        selected_column_index: usize,
    ) -> bool {
        if let Some((x, y)) = self.selection_area_corner {
            let x_in_page = x
                .saturating_add(1)
                .saturating_sub(self.column_page_start.load(Ordering::Relaxed));
            return matches!(
                self.selected_row.selected(),
                Some(selected_row_index)
                if (x_in_page.min(selected_column_index).max(1)..x_in_page.max(selected_column_index) + 1)
                    .contains(&column_index)
                    && (y.min(selected_row_index)..y.max(selected_row_index) + 1)
                        .contains(&row_index)
            );
        }
        matches!(
            self.selected_row.selected(),
            Some(selected_row_index) if row_index == selected_row_index &&  column_index == selected_column_index
        )
    }

    fn is_number_column(&self, row_index: usize, column_index: usize) -> bool {
        matches!(
            self.selected_row.selected(),
            Some(selected_row_index) if row_index == selected_row_index && 0 == column_index
        )
    }

    fn headers(&self, left: usize, right: usize) -> Vec<String> {
        let mut headers = self.headers.clone()[left..right].to_vec();
        headers.insert(0, "".into());
        headers.into_iter().map(|h| h.to_string()).collect()
    }

    fn rows(&self, left: usize, right: usize) -> Vec<Vec<Arc<RwLock<Value>>>> {
        let mut rows = self
            .rows
            .iter()
            .map(|row| row[left..right].to_vec())
            .collect::<Vec<Vec<Arc<RwLock<Value>>>>>();
        // let mut new_rows: Vec<Vec<Value>> =
        //     rows.iter().map(|row| row[left..right].to_vec()).collect();
        for (index, row) in rows.iter_mut().enumerate() {
            row.insert(0, Arc::new(RwLock::new((index + 1).to_string().into())))
        }
        rows
    }

    fn calculate_cell_widths(
        &mut self,
        area_width: u16,
    ) -> (usize, Vec<String>, Vec<Vec<Arc<RwLock<Value>>>>, Vec<Constraint>) {
        let headers = if self.rows.is_empty() && !self.headers.is_empty() { vec![self.headers.iter().map(|h| Arc::new(RwLock::new(h.to_string().into()))).collect::<Vec<Arc<RwLock<Value>>>>().clone()] } else { vec![] };
        let rows = if self.rows.is_empty() && !self.headers.is_empty() { &headers } else { &self.rows };
        if rows.is_empty() {
             return (0, Vec::new(), Vec::new(), Vec::new());
        }
        if self.selected_column_index() < self.column_page_start.load(Ordering::Relaxed) {
            self.column_page_start.store(self.selected_column_index(), Ordering::Relaxed);
        }

        let far_right_column_index = self.selected_column_index();
        let mut column_index = self.selected_column_index();
        let number_column_width = (rows.len() + 1).to_string().width() as u16;
        let mut widths = Vec::new();
        loop {
            let length = rows
                .iter()
                .map(|row| { row.get(column_index).map_or(0, |cell| cell.read().unwrap().width()) })
                .collect::<Vec<usize>>()
                .iter()
                .max()
                .map_or(3, |v| {
                    *v.max(
                        &self.headers
                            .get(column_index)
                            .map_or(3, |header| header.width()),
                    )
                    .clamp(&3, &20)
                });
            if widths.iter().map(|(_, width)| width).sum::<usize>() + length + widths.len() + 1
                >= area_width.saturating_sub(number_column_width) as usize
            {
                column_index += 1;
                break;
            }
            widths.push((self.headers[column_index].name.clone(), length));
            if column_index == self.column_page_start.load(Ordering::Relaxed) {
                break;
            }
            column_index -= 1;
        }
        widths.reverse();

        let far_left_column_index = column_index;
        let selected_column_index = widths.len().saturating_sub(1);
        let mut column_index = far_right_column_index + 1;
        while widths.iter().map(|(_, width)| width).sum::<usize>() + widths.len()
            < area_width.saturating_sub(number_column_width) as usize
        {
            let length = rows
                .iter()
                .map(|row| { row.get(column_index).map_or(0, |cell| cell.read().unwrap().width()) })
                .collect::<Vec<usize>>()
                .iter()
                .max()
                .map_or(3, |v| {
                    *v.max(
                        self.headers
                            .iter()
                            .map(|header| header.width())
                            .collect::<Vec<usize>>()
                            .get(column_index)
                            .unwrap_or(&3),
                    )
                    .clamp(&3, &20)
                });
            match self.headers.get(column_index) {
                Some(header) => {
                    widths.push((header.to_string(), length));
                }
                None => break,
            }
            column_index += 1
        }
        if self.selected_column_index() != self.headers.len().saturating_sub(1)
            && column_index.saturating_sub(1) != self.headers.len().saturating_sub(1)
        {
            widths.pop();
        }
        let far_right_column_index = column_index;
        let mut constraints = widths
            .iter()
            .map(|(_, width)| Constraint::Length(*width as u16))
            .collect::<Vec<Constraint>>();
        if self.selected_column_index() != self.headers.len().saturating_sub(1)
            && column_index.saturating_sub(1) != self.headers.len().saturating_sub(1)
        {
            constraints.push(Constraint::Min(10));
        }
        constraints.insert(0, Constraint::Length(number_column_width));
        for (i, adjust) in self.constraint_adjust[far_left_column_index..far_right_column_index].iter().enumerate() {
            if *adjust > 0 {
                match constraints[i] {
                    Constraint::Length(l) => { constraints[i] = Constraint::Length(l+*adjust) },
                    _ => {}
                }
            }
        }
        self.column_page_start.store(far_left_column_index, Ordering::Relaxed);

        (
            self.selection_area_corner
                .map_or(selected_column_index + 1, |(x, _)| {
                    if x > self.selected_column {
                        (selected_column_index + 1)
                            .saturating_sub(x.saturating_sub(self.selected_column))
                    } else {
                        (selected_column_index + 1)
                            .saturating_add(self.selected_column.saturating_sub(x))
                    }
                }),
            self.headers(far_left_column_index, far_right_column_index),
            self.rows(far_left_column_index, far_right_column_index),
            constraints,
        )
    }

    // column order: primary_key,id,first column
    async fn primary_key_value(&self, pool: &Box<dyn Pool>, database: &Database, table: &DTable) -> anyhow::Result<(String, Vec<Value>)> {
        let database_type = pool.database_type();
        let columns = database_type.primary_key_columns(pool, &database, &table).await?;
        if  let Some(primary_key) = columns.iter().next() {
            if let Some(index) = self.headers.iter().position(|h| h.name == *primary_key) {
                if let Some(value) = self.selected_rows()
                        .map(|rows| rows.iter().filter_map(|row| row.get(index).map(|s| s.clone()))
                        .map(|v| v.read().unwrap().clone())
                        .collect::<Vec<_>>()) {
                    return Ok((primary_key.to_string(), value))
                }
            }
        } else {
            if let Some((index, col)) = self.headers.iter().enumerate().find(|(_, h)| h.name.to_lowercase() == "id" ).or(self.headers.iter().enumerate().next()) {
                if let Some(value) = self.selected_rows()
                        .map(|rows| rows.iter().filter_map(|row| row.get(index).map(|s| s.clone()))
                        .map(|v| v.read().unwrap().clone())
                        .collect::<Vec<_>>()) {
                    return Ok((col.name.clone(), value))
                }
                // if let Some(id) = self.selected_rows().map(|rows| rows.iter().map(|row| row.get(index).map(|s| s.clone()))).flatten().flatten() {
                //     return Ok((col.name.clone(), id.read().unwrap().clone()))
                // }
            }
        }
        anyhow::bail!("primary key not found")
    }

    async fn dispatch_command(&self, command: &str, store: &Store) -> anyhow::Result<()>  {
        if command == "tree" {
            store.dispatch(Event::ToggleTree).await?;
        }
        Ok(())
    }

}

impl StatefulDrawableComponent for TableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let chunks = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .direction(Direction::Vertical)
            .constraints(
                [
                    // Constraint::Length(2),
                    Constraint::Min(1),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(area);

        f.render_widget(
            Block::default()
                .title(self.title())
                .borders(Borders::ALL)
                .style(if focused {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
            area,
        );

        self.selected_row.selected().map_or_else(
            || {
                self.scroll.reset();
            },
            |selection| {
                self.scroll.update(
                    selection,
                    self.rows.len(),
                    chunks[0].height.saturating_sub(2) as usize,
                );
            },
        );

        let block = Block::default().borders(Borders::NONE);
        self.area_width = block.inner(chunks[1]).width;
        let (selected_column_index, headers, rows, constraints) =
            self.calculate_cell_widths(self.area_width);
        let header_cells = headers.iter().enumerate().map(|(column_index, h)| {
            Cell::from(h.to_string()).style(if selected_column_index == column_index {
                Style::default().fg(self.settings.color).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
        });
        let header = Row::new(header_cells).height(1).bottom_margin(1);
        let rows = rows.iter().enumerate().map(|(row_index, item)| {
            let height = item
                .iter()
                .map(|content| content.read().unwrap().data.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().enumerate().map(|(column_index, c)| {
                let c = c.read().unwrap();
                let value = if c.is_null { format!("<{}>", "NULL") } else { c.substr(256) };
                let style = if self.is_selected_cell(row_index, column_index, selected_column_index) {
                    Style::default().bg(self.settings.color)
                } else if self.is_number_column(row_index, column_index) {
                    Style::default().add_modifier(Modifier::BOLD)
                } else if c.is_null {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                Cell::from(value).style(style)
            });
            Row::new(cells).height(height as u16).bottom_margin(1)
        });

        let table = Table::new(rows)
            .header(header)
            .block(block)
            .style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .widths(&constraints);
        let mut state = self.selected_row.clone();
        f.render_stateful_widget(
            table,
            chunks[0],
            if let Some((_, y)) = self.selection_area_corner {
                state.select(Some(y));
                &mut state
            } else {
                &mut self.selected_row
            },
        );

        // TableValueComponent::new(self.selected_cells().unwrap_or_default())
        //     .draw(f, chunks[0], focused)?;

        match self.focus {
            Focus::Status => {
                TableStatusComponent::new(
                    if self.rows.is_empty() {
                        None
                    } else {
                        Some(self.rows.len())
                    },
                    if self.headers.is_empty() {
                        None
                    } else {
                        Some(self.headers.len())
                    },
                    self.selected_cells(),
                    self.table.as_ref().map(|t| t.1.clone()),
                )
                .draw(f, chunks[1], focused)?;
            }
            Focus::Command => { self.command_editor.draw(f, chunks[1], focused)?; },
            _ => {
                self.cell_editor.draw(f, chunks[1], focused)?;
            },
        };

        self.scroll.draw(f, chunks[0]);
        Ok(())
    }
}

#[async_trait]
impl Component for TableComponent {
    fn helps(&self, out: &mut Vec<HelpInfo>) {
        out.push(HelpInfo::new(help_info::extend_selection_by_one_cell(
            &self.key_config,
        )));
    }

    fn event(&mut self, key: &[Key]) -> Result<EventState> {
        if self.focus == Focus::Editor {
            let state = self.cell_editor.event(key)?;
            if state == EventState::Consumed {
                return Ok(EventState::Consumed)
            }
        }
        if self.focus == Focus::Command {
            let state = self.command_editor.event(key)?;
            if state == EventState::Consumed {
                return Ok(EventState::Consumed)
            }
        }
        if key == self.key_config.copy2 {
            let header = &self.headers[self.selected_column];
            copy_to_clipboard(&header.name)?;
            return Ok(EventState::Consumed);
        // } else if key == self.key_config.reset_column_width {
        //     self.reset_column();
        //     return Ok(EventState::Consumed);
        } else if key == [self.key_config.jump_to_start] {
            self.first_column();
            return Ok(EventState::Consumed);
        } else if key == [self.key_config.jump_to_end] {
            self.last_column();
            return Ok(EventState::Consumed);
        } else if key == [self.key_config.repeat_movement] {
            if let Some(movement) = self.movement {
                match movement {
                    Movement::Forward(c) => self.forward_by_character(c),
                    Movement::Backward(c) => self.backward_by_character(c),
                }
            }
            return Ok(EventState::Consumed);
        } else if key[0] == self.key_config.forward && key.len() > 1 {
            match key[1] {
                Key::Char(c) => self.forward_by_character(c),
                _ => {},
            };
            return Ok(EventState::Consumed);
        } else if key[0] == self.key_config.backward && key.len() > 1 {
            match key[1] {
                Key::Char(c) => self.backward_by_character(c),
                _ => {},
            };
            return Ok(EventState::Consumed);
        }
        
        let key = key[0];
        if key == self.key_config.scroll_left {
            self.previous_column();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down {
            self.next_row(1);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next_row(10);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_up {
            self.previous_row(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous_row(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_top {
            self.scroll_to_top();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_bottom {
            self.scroll_to_bottom();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_right {
            self.next_column();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.extend_selection_by_one_cell_left {
            self.expand_selected_area_x(false);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.extend_selection_by_one_cell_up {
            self.expand_selected_area_y(false);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.extend_selection_by_one_cell_down {
            self.expand_selected_area_y(true);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.extend_selection_by_one_cell_right {
            self.expand_selected_area_x(true);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.expand_column_width {
            self.expand_column();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.shorten_column_width {
            self.shorten_column();
            return Ok(EventState::Consumed);
        // } else if key == self.key_config.reset_column_width {
        //     self.reset_column();
        //     return Ok(EventState::Consumed);
        // } else if key == self.key_config.jump_to_start {
        //     self.first_column();
        //     return Ok(EventState::Consumed);
        // } else if key == self.key_config.jump_to_end {
        //     self.last_column();
        //     return Ok(EventState::Consumed);
        } else if key == self.key_config.edit_cell && self.focus == Focus::Status {
            self.focus = Focus::Editor;
            let s = self.selected_cell().map(|c| if c.is_null { NULL.to_string() } else { c.to_string() });
            self.cell_editor.update(s.unwrap_or("".to_string()));
            return Ok(EventState::Consumed);
        } else if key == self.key_config.edit_command && self.focus == Focus::Status {
            self.focus = Focus::Command;
            return Ok(EventState::Consumed);
        } else if key == self.key_config.exit_popup {
            self.focus = Focus::Status;
            return Ok(EventState::Consumed);
        }
        
        Ok(EventState::NotConsumed)
    }

    async fn async_event(
        &mut self,
        key: crate::event::Key,
        pool: &Box<dyn Pool>,
        store: &Store
    ) -> Result<EventState> {
        // delete by primary_key
        if key == self.key_config.delete {
            if let Some((database, table)) = &self.table {
                let (primary_key, values) = self.primary_key_value(pool, database, table).await?;
                let col_values = values.iter().map(|v| v.data.as_str()).collect::<Vec<_>>();
                // let sql = pool.database_type().delete_row_by_column(&database, &table, &primary_key, &values[0].data);
                let sql = pool.database_type().delete_rows_by_column(&database, &table, &primary_key, &col_values);
                pool.execute(&sql).await?;
                store.dispatch(Event::RedrawTable(true)).await?;
                return Ok(EventState::Consumed)
            }
        }
        if key == self.key_config.advanced_copy {
            if let Some(rows) = self.selected_rows() {
                if let Some((database, table)) = &self.table {
                    let sql = pool.database_type().insert_rows(database, table, &self.headers, &rows);
                    copy_to_clipboard(sql.trim())?;
                }
            }
            return Ok(EventState::Consumed);
        }

        if key == self.key_config.orderby_desc {
            let header = &self.headers[self.selected_column];
            let mut orderby = format!("{} desc", header.name);
            if self.orderby_status.as_deref() == Some(&orderby) {
                orderby = String::new();
                self.orderby_status = None;
            } else { self.orderby_status = Some(orderby.clone()) };
            store.dispatch(Event::OrderByTable((orderby, self.selected_column))).await?;
            return Ok(EventState::Consumed);
        }

        if key == self.key_config.orderby_asc {
            let header = &self.headers[self.selected_column];
            let mut orderby = format!("{} asc", header.name);
            if self.orderby_status.as_deref() == Some(&orderby) {
                orderby = String::new();
                self.orderby_status = None;
            } else { self.orderby_status = Some(orderby.clone()) };
            store.dispatch(Event::OrderByTable((orderby, self.selected_column))).await?;
            return Ok(EventState::Consumed);
        }

        // update cell value
        if key == self.key_config.enter && self.focus == Focus::Editor {
            self.focus = Focus::Status;
            if let Some((database, table)) = &self.table {
                let (pkey, pval) = self.primary_key_value(pool, database, table).await?;
                let header = &self.headers[self.selected_column];
                let v = self.cell_editor.value();
                let value = if v == NULL { Value::default() } else { Value::new(v.clone()) };
                let sql = pool.database_type().update_row_by_column(database, table, &pkey, &pval[0].data, &header, &value);
                self.set_selected_cell(v);
                pool.execute(&sql).await?;
                return Ok(EventState::Consumed)
            }
        }
        // execute command
        if key == self.key_config.enter && self.focus == Focus::Command {
            self.focus = Focus::Status;
            let command = self.command_editor.value();
            self.command_editor.reset();
            self.dispatch_command(command.trim(), store).await?;
            return Ok(EventState::Consumed)
        }
        Ok(EventState::NotConsumed)
    }
}

#[cfg(test)]
mod test {
    use super::{KeyConfig, Settings, TableComponent};
    use tui::layout::Constraint;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_headers() {
        let mut component = TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["a", "b", "c"].into_iter().map(|h| h.into()).collect();
        assert_eq!(component.headers(1, 2), vec!["", "b"])
    }

    #[test]
    fn test_rows() {
        let mut component = TableComponent::new(KeyConfig::default(), Settings::default());
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        let rows = component.rows(1, 2).iter().map(|row|  row.iter().map(|cell|  cell.read().unwrap().clone()).collect::<Vec<_>>()).collect::<Vec<_>>();
        assert_eq!(rows, vec![vec!["1", "b"], vec!["2", "e"]],)
    }

    #[test]
    fn test_expand_selected_area_x_left() {
        // before
        //    1  2  3
        // 1  a  b  c
        // 2  d |e| f

        // after
        //    1  2  3
        // 1  a  b  c
        // 2 |d  e| f

        let mut component =  TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        component.selected_row.select(Some(1));
        component.selected_column = 1;
        component.expand_selected_area_x(false);
        assert_eq!(component.selection_area_corner, Some((0, 1)));
        assert_eq!(component.selected_cells(), Some("d,e".to_string()));
    }

    #[test]
    fn test_expand_selected_area_x_right() {
        // before
        //    1  2  3
        // 1  a  b  c
        // 2  d |e| f

        // after
        //    1  2  3
        // 1  a  b  c
        // 2  d |e  f|

        let mut component =  TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        component.selected_row.select(Some(1));
        component.selected_column = 1;
        component.expand_selected_area_x(true);
        assert_eq!(component.selection_area_corner, Some((2, 1)));
        assert_eq!(component.selected_cells(), Some("e,f".to_string()));
    }

    #[test]
    fn test_expand_selected_area_y_up() {
        // before
        //    1  2  3
        // 1  a  b  c
        // 2  d |e| f

        // after
        //    1  2  3
        // 1  a |b| c
        // 2  d |e| f

        let mut component =  TableComponent::new(KeyConfig::default(), Settings::default());
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        component.selected_row.select(Some(1));
        component.selected_column = 1;
        component.expand_selected_area_y(false);
        assert_eq!(component.selection_area_corner, Some((1, 0)));
        assert_eq!(component.selected_cells(), Some("b\ne".to_string()));
    }

    #[test]
    fn test_expand_selected_area_y_down() {
        // before
        //    1  2  3
        // 1  a |b| c
        // 2  d  e  f

        // after
        //    1  2  3
        // 1  a |b| c
        // 2  d |e| f

        let mut component =  TableComponent::new(KeyConfig::default(), Settings::default());
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        component.selected_row.select(Some(0));
        component.selected_column = 1;
        component.expand_selected_area_y(true);
        assert_eq!(component.selection_area_corner, Some((1, 1)));
        assert_eq!(component.selected_cells(), Some("b\ne".to_string()));
    }

    #[test]
    fn test_is_number_column() {
        let mut component =  TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        component.selected_row.select(Some(0));
        assert!(component.is_number_column(0, 0));
        assert!(!component.is_number_column(0, 1));
    }

    #[test]
    fn test_selected_cell_when_one_cell_selected() {
        //    1  2 3
        // 1 |a| b c
        // 2  d  e f

        let mut component = TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        component.selected_row.select(Some(0));
        assert_eq!(component.selected_cells(), Some("a".to_string()));
    }

    #[test]
    fn test_selected_cell_when_multiple_cells_selected() {
        //    1  2  3
        // 1 |a  b| c
        // 2 |d  e| f

        let mut component = TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        component.selected_row.select(Some(0));
        component.selection_area_corner = Some((1, 1));
        assert_eq!(component.selected_cells(), Some("a,b\nd,e".to_string()));
    }

    #[test]
    fn test_is_selected_cell_when_one_cell_selected() {
        //    1  2 3
        // 1 |a| b c
        // 2  d  e f

        let mut component = TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        component.selected_row.select(Some(0));
        // a
        assert!(component.is_selected_cell(0, 1, 1));
        // d
        assert!(!component.is_selected_cell(1, 1, 1));
        // e
        assert!(!component.is_selected_cell(1, 2, 1));
    }

    #[test]
    fn test_is_selected_cell_when_multiple_cells_selected() {
        //    1  2  3
        // 1 |a  b| c
        // 2 |d  e| f

        let mut component = TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        component.selected_row.select(Some(0));
        component.selection_area_corner = Some((1, 1));
        // a
        assert!(component.is_selected_cell(0, 1, 1));
        // b
        assert!(component.is_selected_cell(0, 2, 1));
        // d
        assert!(component.is_selected_cell(1, 1, 1));
        // e
        assert!(component.is_selected_cell(1, 2, 1));
        // f
        assert!(!component.is_selected_cell(1, 3, 1));
    }

    #[test]
    fn test_calculate_cell_widths_when_sum_of_cell_widths_is_greater_than_table_width() {
        let mut component = TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.constraint_adjust= vec![0, 0, 0];
        component.rows = vec![
            vec!["aaaaa", "bbbbb", "ccccc"]
                .iter()
                .map(|h| Arc::new(RwLock::new(h.into())))
                .collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];
        let (selected_column_index, headers, rows, constraints) =
            component.calculate_cell_widths(10);
        assert_eq!(selected_column_index, 1);
        assert_eq!(headers, vec!["", "1", "2"]);
        let rows = rows.iter().map(|row|  row.iter().map(|cell|  cell.read().unwrap().clone()).collect::<Vec<_>>()).collect::<Vec<_>>();
        assert_eq!(rows, vec![vec!["1", "aaaaa", "bbbbb"], vec!["2", "d", "e"]]);
        assert_eq!(
            constraints,
            vec![
                Constraint::Length(1),
                Constraint::Length(5),
                Constraint::Min(10),
            ]
        );
    }

    #[test]
    fn test_calculate_cell_widths_when_sum_of_cell_widths_is_less_than_table_width() {
        let mut component = TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.constraint_adjust= vec![0, 0, 0];
        component.rows = vec![
            vec!["aaaaa", "bbbbb", "ccccc"]
                .iter()
                .map(|h| Arc::new(RwLock::new(h.into())))
                .collect(),
            vec!["d", "e", "f"].iter().map(|h| Arc::new(RwLock::new(h.into()))).collect(),
        ];

        let (selected_column_index, headers, rows, constraints) =
            component.calculate_cell_widths(20);
        assert_eq!(selected_column_index, 1);
        assert_eq!(headers, vec!["", "1", "2", "3"]);
        let rows = rows.iter().map(|row|  row.iter().map(|cell|  cell.read().unwrap().clone()).collect::<Vec<_>>()).collect::<Vec<_>>();
        assert_eq!(
            rows,
            vec![
                vec!["1", "aaaaa", "bbbbb", "ccccc"],
                vec!["2", "d", "e", "f"]
            ]
        );
        assert_eq!(
            constraints,
            vec![
                Constraint::Length(1),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
            ]
        );
    }

    #[test]
    fn test_calculate_cell_widths_when_component_has_multiple_rows() {
        let mut component = TableComponent::new(KeyConfig::default(), Settings::default());
        component.headers = vec!["1", "2", "3"].into_iter().map(|h| h.into()).collect();
        component.constraint_adjust= vec![0, 0, 0];
        component.rows = vec![
            vec!["aaaaa", "bbbbb", "ccccc"]
                .iter()
                .map(|h| Arc::new(RwLock::new(h.into())))
                .collect(),
            vec!["dddddddddd", "e", "f"]
                .iter()
                .map(|h| Arc::new(RwLock::new(h.into())))
                .collect(),
        ];

        let (selected_column_index, headers, rows, constraints) =
            component.calculate_cell_widths(20);
        assert_eq!(selected_column_index, 1);
        assert_eq!(headers, vec!["", "1", "2", "3"]);
        let rows = rows.iter().map(|row|  row.iter().map(|cell|  cell.read().unwrap().clone()).collect::<Vec<_>>()).collect::<Vec<_>>();
        assert_eq!(
            rows,
            vec![
                vec!["1", "aaaaa", "bbbbb", "ccccc"],
                vec!["2", "dddddddddd", "e", "f"]
            ]
        );
        assert_eq!(
            constraints,
            vec![
                Constraint::Length(1),
                Constraint::Length(10),
                Constraint::Length(5),
                Constraint::Length(5),
            ]
        );
    }
}
