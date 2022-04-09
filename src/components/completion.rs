use super::{Component, EventState, MovableComponent};
use crate::components::help_info::CommandInfo;
use crate::config::{KeyConfig, Settings};
use crate::event::Key;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

const RESERVED_WORDS_IN_WHERE_CLAUSE: &[&str] = &["IN", "AND", "OR", "NOT", "NULL", "IS"];
const ALL_RESERVED_WORDS: &[&str] = &[
    "IN", "AND", "OR", "NOT", "NULL", "IS", "SELECT", "UPDATE", "DELETE", "FROM", "LIMIT", "WHERE",
];

pub struct CompletionComponent {
    key_config: KeyConfig,
    settings: Settings,
    state: ListState,
    word: String,
    candidates: Vec<String>,
    visible: bool,
}

impl CompletionComponent {
    pub fn new(key_config: KeyConfig, settings: Settings, word: impl Into<String>, all: bool) -> Self {
        Self {
            key_config,
            settings,
            state: ListState::default(),
            word: word.into(),
            candidates: if all {
                ALL_RESERVED_WORDS.iter().map(|w| w.to_string()).collect()
            } else {
                RESERVED_WORDS_IN_WHERE_CLAUSE
                    .iter()
                    .map(|w| w.to_string())
                    .collect()
            },
            visible: false, 
        }
    }

    pub fn new_with_candidates(key_config: KeyConfig, settings: Settings, candidates: Vec<&str>) -> Self {
        let mut candidates: Vec<_> = candidates.iter().map(|w| w.to_string()).collect();
        candidates.sort();
        candidates.dedup();
        Self {
            key_config,
            settings,
            state: ListState::default(),
            word: "".to_string(),
            candidates,
            visible: false,
        }
    }

    pub fn update(&mut self, word: impl Into<String>) {
        self.word = word.into();
        self.visible = false;
        self.state.select(None);
        self.state.select(Some(0))
    }

    pub fn update_candidates(&mut self, candidates: &[String]) {
        for candidate in candidates {
            if self.candidates.iter().find(|x| *x == candidate).is_none() {
                self.candidates.push(candidate.clone())
            }
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filterd_candidates().count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filterd_candidates().count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn filterd_candidates(&self) -> impl Iterator<Item = &String> {
        self.candidates.iter().filter(move |c| {
            (c.starts_with(self.word.to_lowercase().as_str())
                || c.starts_with(self.word.to_uppercase().as_str()))
                && !self.word.is_empty()
        })
    }

    pub fn selected_candidate(&self) -> Option<String> {
        self.filterd_candidates()
            .collect::<Vec<&String>>()
            .get(self.state.selected()?)
            .map(|c| c.to_string())
    }

    pub fn word(&self) -> String {
        self.word.to_string()
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}

impl MovableComponent for CompletionComponent {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        area: Rect,
        _focused: bool,
        x: u16,
        y: u16,
    ) -> Result<()> {
        if !self.word.is_empty() {
            let width = 30;
            let candidates = self
                .filterd_candidates()
                .map(|c| ListItem::new(c.to_string()))
                .collect::<Vec<ListItem>>();
            let candidates_len = candidates.len(); 
            if candidates_len == 0 {
                self.visible = false;
                return Ok(());
            }
            self.visible = true;
            let candidate_list = List::new(candidates)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().bg(self.settings.color))
                .style(Style::default());

            let area = Rect::new(
                area.x + x,
                area.y + y + 2,
                width
                    .min(f.size().width)
                    .min(f.size().right().saturating_sub(area.x + x)),
                (candidates_len.min(5) as u16 + 2)
                    .min(f.size().bottom().saturating_sub(area.y + y + 2)),
            );
            f.render_widget(Clear, area);
            f.render_stateful_widget(candidate_list, area, &mut self.state);
        }
        Ok(())
    }
}

impl Component for CompletionComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: &[Key]) -> Result<EventState> {
        let key = key[0];
        if key == self.key_config.move_down {
            self.next();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.move_up {
            self.previous();
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}

#[cfg(test)]
mod test {
    use super::{CompletionComponent, KeyConfig, Settings};

    #[test]
    fn test_filterd_candidates_lowercase() {
        assert_eq!(
            CompletionComponent::new(KeyConfig::default(), Settings::default(),"an", false)
                .filterd_candidates()
                .collect::<Vec<&String>>(),
            vec![&"AND".to_string()]
        );
    }

    #[test]
    fn test_filterd_candidates_uppercase() {
        assert_eq!(
            CompletionComponent::new(KeyConfig::default(), Settings::default(), "AN", false)
                .filterd_candidates()
                .collect::<Vec<&String>>(),
            vec![&"AND".to_string()]
        );
    }

    #[test]
    fn test_filterd_candidates_multiple_candidates() {
        assert_eq!(
            CompletionComponent::new(KeyConfig::default(), Settings::default(), "n", false)
                .filterd_candidates()
                .collect::<Vec<&String>>(),
            vec![&"NOT".to_string(), &"NULL".to_string()]
        );

        assert_eq!(
            CompletionComponent::new(KeyConfig::default(), Settings::default(), "N", false)
                .filterd_candidates()
                .collect::<Vec<&String>>(),
            vec![&"NOT".to_string(), &"NULL".to_string()]
        );
    }
}
