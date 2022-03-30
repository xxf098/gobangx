use tui::{backend::Backend, layout::Rect, Frame};
use tui::widgets::{Wrap, Block, Borders};
use unicode_width::UnicodeWidthStr;
use super::{compute_character_width, Component, EventState, StatefulDrawableComponent};
use crate::components::command::CommandInfo;
use crate::event::Key;
use crate::ui::stateful_paragraph::{ParagraphState, StatefulParagraph};


pub struct CellEditorComponent {
    input: Vec<char>,
    input_idx: usize,
    paragraph_state: ParagraphState,
    input_cursor_position_x: u16,
}

impl CellEditorComponent {

    pub fn new(input: String) -> Self {
        let input_cursor_position_x = input.len() as u16;
        Self { 
            input_idx: 0,
            input: input.chars().collect(),
            paragraph_state: ParagraphState::default(),
            input_cursor_position_x,
        }
    }

    pub fn update(&mut self, input: String) {
        self.input_idx = input.len();
        self.input_cursor_position_x = self.input_idx as u16;
        self.input = input.chars().collect();
    }
}


impl StatefulDrawableComponent for CellEditorComponent {

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, _focused: bool) -> anyhow::Result<()> {
        let editor = StatefulParagraph::new(self.input.iter().collect::<String>())
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::TOP));
        f.render_stateful_widget(editor, area, &mut self.paragraph_state);
        let x = area.x
        .saturating_add(
            self.input_cursor_position_x % area.width.saturating_sub(2),
        )
        .min(area.right().saturating_sub(2));
        f.set_cursor(x, area.y+1);
        Ok(())
    }
}

impl Component for CellEditorComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        let input_str: String = self.input.iter().collect();
        match key {
            Key::Char(c) => {
                self.input.insert(self.input_idx, c);
                self.input_idx += 1;
                self.input_cursor_position_x += 1;
                return Ok(EventState::Consumed);
            }
            Key::Delete | Key::Backspace  => {
                if input_str.width() > 0 && !self.input.is_empty() && self.input_idx > 0 {
                    self.input.remove(self.input_idx - 1);
                    self.input_idx -= 1;
                    self.input_cursor_position_x -= 1;
                }
                return Ok(EventState::Consumed);
            }
            Key::Left => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx -= 1;
                    self.input_cursor_position_x = self
                        .input_cursor_position_x
                        .saturating_sub(compute_character_width(self.input[self.input_idx]));
                }
                return Ok(EventState::Consumed);
            }
            Key::Right => {
                if self.input_idx < self.input.len() {
                    let next_c = self.input[self.input_idx];
                    self.input_idx += 1;
                    self.input_cursor_position_x += compute_character_width(next_c);
                }
                return Ok(EventState::Consumed);
            }
            _ => (),
        }
        return Ok(EventState::NotConsumed);
    }
}