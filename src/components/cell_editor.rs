use tui::{backend::Backend, layout::Rect, Frame};
use tui::widgets::{Wrap, Block, Borders};
use super::{Component, EventState, StatefulDrawableComponent};
use crate::components::command::CommandInfo;
use crate::config::{KeyConfig};
use crate::event::Key;
use crate::ui::stateful_paragraph::{ParagraphState, StatefulParagraph};


pub struct CellEditorComponent {
    key_config: KeyConfig,
    value: String,
    paragraph_state: ParagraphState,
}

impl CellEditorComponent {

    pub fn new(key_config: KeyConfig, value: String) -> Self {
        Self { 
            key_config, 
            value: value,
            paragraph_state: ParagraphState::default(), 
        }
    }
}


impl StatefulDrawableComponent for CellEditorComponent {

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, _focused: bool) -> anyhow::Result<()> {
        let editor = StatefulParagraph::new(self.value.clone())
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::TOP));
        f.render_stateful_widget(editor, area, &mut self.paragraph_state);
        Ok(())
    }
}

impl Component for CellEditorComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.move_down {
            return Ok(EventState::Consumed);
        } else if key == self.key_config.move_up {
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}