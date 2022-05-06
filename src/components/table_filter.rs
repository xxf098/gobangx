use super::{
    compute_character_width, PlainCompletionComponent, Component, EventState, MovableComponent,
    StatefulDrawableComponent,
};
use crate::components::help_info::HelpInfo;
use crate::config::{KeyConfig, Settings};
use crate::event::Key;
use anyhow::Result;
use database_tree::Table;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub struct TableFilterComponent {
    key_config: KeyConfig,
    settings: Settings,
    pub table: Option<Table>,
    input: Vec<char>,
    input_idx: usize,
    input_cursor_position: u16,
    completion: PlainCompletionComponent,
}

impl TableFilterComponent {
    pub fn new(key_config: KeyConfig, settings: Settings) -> Self {
        Self {
            key_config: key_config.clone(),
            table: None,
            input: Vec::new(),
            input_idx: 0,
            input_cursor_position: 0,
            completion: PlainCompletionComponent::new(key_config, settings.clone(),"", false),
            settings,
        }
    }

    pub fn input_str(&self) -> String {
        self.input.iter().collect()
    }

    pub fn reset(&mut self) {
        self.table = None;
        self.input = Vec::new();
        self.input_idx = 0;
        self.input_cursor_position = 0;
    }

    pub fn update_candidates(&mut self, candidates: &[String]) {
        self.completion.update_candidates(candidates, None)
    }

    fn update_completion(&mut self) {
        let input = &self
            .input
            .iter()
            .enumerate()
            .filter(|(i, _)| i < &self.input_idx)
            .map(|(_, i)| i)
            .collect::<String>()
            .split(' ')
            .map(|i| i.to_string())
            .collect::<Vec<String>>();
        let full_text: String = self.input.iter().collect();
        self.completion
            .update(input.last().unwrap_or(&String::new()), full_text);
    }

    fn complete(&mut self) -> anyhow::Result<EventState> {
        if let Some(candidate) = self.completion.selected_candidate() {
            let mut input = Vec::new();
            let first = self
                .input
                .iter()
                .enumerate()
                .filter(|(i, _)| i < &self.input_idx.saturating_sub(self.completion.word().len()))
                .map(|(_, c)| c.to_string())
                .collect::<Vec<String>>();
            let last = self
                .input
                .iter()
                .enumerate()
                .filter(|(i, _)| i >= &self.input_idx)
                .map(|(_, c)| c.to_string())
                .collect::<Vec<String>>();

            let is_last_word = last.first().map_or(false, |c| c == &" ".to_string());

            let middle = if is_last_word {
                candidate
                    .chars()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
            } else {
                let mut c = candidate
                    .chars()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>();
                c.push(" ".to_string());
                c
            };

            input.extend(first);
            input.extend(middle.clone());
            input.extend(last);

            self.input = input.join("").chars().collect();
            self.input_idx += &middle.len();
            if is_last_word {
                self.input_idx += 1;
            }
            self.input_idx -= self.completion.word().len();
            self.input_cursor_position += middle
                .join("")
                .chars()
                .map(compute_character_width)
                .sum::<u16>();
            if is_last_word {
                self.input_cursor_position += " ".to_string().width() as u16
            }
            self.input_cursor_position -= self
                .completion
                .word()
                .chars()
                .map(compute_character_width)
                .sum::<u16>();
            self.update_completion();
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}

impl StatefulDrawableComponent for TableFilterComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let query = Paragraph::new(Spans::from(vec![
            Span::styled(
                self.table
                    .as_ref()
                    .map_or("-".to_string(), |table| table.name.to_string()),
                Style::default().fg(self.settings.color),
            ),
            Span::from(format!(
                " {}",
                if focused || !self.input.is_empty() {
                    self.input.iter().collect::<String>()
                } else {
                    "Enter a SQL expression in WHERE clause to filter records".to_string()
                }
            )),
        ]))
        .style(if focused {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(query, area);

        if focused {
            self.completion.draw(
                f,
                area,
                false,
                (self
                    .table
                    .as_ref()
                    .map_or(String::new(), |table| {
                        format!("{} ", table.name.to_string())
                    })
                    .width() as u16)
                    .saturating_add(self.input_cursor_position),
                0,
            )?;
        };

        if focused {
            f.set_cursor(
                (area.x
                    + (1 + self
                        .table
                        .as_ref()
                        .map_or(String::new(), |table| table.name.to_string())
                        .width()
                        + 1) as u16)
                    .saturating_add(self.input_cursor_position)
                    .min(area.right().saturating_sub(2)),
                area.y + 1,
            )
        }
        Ok(())
    }
}

impl Component for TableFilterComponent {
    fn helps(&self, _out: &mut Vec<HelpInfo>) {}

    fn event(&mut self, key: &[Key]) -> Result<EventState> {
        let input_str: String = self.input.iter().collect();

        // apply comletion candidates
        if key[0] == self.key_config.enter {
            return self.complete();
        }

        self.completion.selected_candidate();

        match key {
            [Key::Char(c)] => {
                self.input.insert(self.input_idx, *c);
                self.input_idx += 1;
                self.input_cursor_position += compute_character_width(*c);
                self.update_completion();

                Ok(EventState::Consumed)
            }
            [Key::Delete | Key::Backspace] => {
                if input_str.width() > 0 && !self.input.is_empty() && self.input_idx > 0 {
                    let last_c = self.input.remove(self.input_idx - 1);
                    self.input_idx -= 1;
                    self.input_cursor_position -= compute_character_width(last_c);
                    self.completion.update("", "");
                }
                Ok(EventState::Consumed)
            }
            [Key::Left] => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx -= 1;
                    self.input_cursor_position = self
                        .input_cursor_position
                        .saturating_sub(compute_character_width(self.input[self.input_idx]));
                    self.completion.update("", "");
                }
                Ok(EventState::Consumed)
            }
            [Key::Ctrl('a')] => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx = 0;
                    self.input_cursor_position = 0
                }
                Ok(EventState::Consumed)
            }
            [Key::Right] => {
                if self.input_idx < self.input.len() {
                    let next_c = self.input[self.input_idx];
                    self.input_idx += 1;
                    self.input_cursor_position += compute_character_width(next_c);
                    self.completion.update("", "");
                }
                Ok(EventState::Consumed)
            }
            [Key::Ctrl('e')] => {
                if self.input_idx < self.input.len() {
                    self.input_idx = self.input.len();
                    self.input_cursor_position = self.input_str().width() as u16;
                }
                Ok(EventState::Consumed)
            }
            key => self.completion.event(key),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{KeyConfig, TableFilterComponent, Settings};

    #[test]
    fn test_complete() {
        let mut filter = TableFilterComponent::new(KeyConfig::default(), Settings::default());
        filter.input_idx = 2;
        filter.input = vec!['a', 'n', ' ', 'c', 'd', 'e', 'f', 'g'];
        filter.completion.update("an", "");
        assert!(filter.complete().is_ok());
        assert_eq!(
            filter.input,
            vec!['A', 'N', 'D', ' ', 'c', 'd', 'e', 'f', 'g']
        );
    }

    #[test]
    fn test_complete_end() {
        let mut filter = TableFilterComponent::new(KeyConfig::default(), Settings::default());
        filter.input_idx = 9;
        filter.input = vec!['a', 'b', ' ', 'c', 'd', 'e', 'f', ' ', 'i'];
        filter.completion.update('i', "");
        assert!(filter.complete().is_ok());
        assert_eq!(
            filter.input,
            vec!['a', 'b', ' ', 'c', 'd', 'e', 'f', ' ', 'I', 'N', ' ']
        );
    }

    #[test]
    fn test_complete_no_candidates() {
        let mut filter = TableFilterComponent::new(KeyConfig::default(), Settings::default());
        filter.input_idx = 2;
        filter.input = vec!['a', 'n', ' ', 'c', 'd', 'e', 'f', 'g'];
        filter.completion.update("foo", "");
        assert!(filter.complete().is_ok());
        assert_eq!(filter.input, vec!['a', 'n', ' ', 'c', 'd', 'e', 'f', 'g']);
    }
}
