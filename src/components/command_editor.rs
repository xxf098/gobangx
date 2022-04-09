use tui::{backend::Backend, layout::Rect, Frame};
use super::{LineEditorComponent, StatefulDrawableComponent, Component, EventState};
use crate::event::Key;

pub struct CommandEditorComponent {
    editor: LineEditorComponent,
}


impl CommandEditorComponent {

    pub fn new(input: String) -> Self {
        Self {
            editor: LineEditorComponent::new(input),
        }
    }

    pub fn value(&self) -> String {
        self.editor.value()
    }
}

impl StatefulDrawableComponent for CommandEditorComponent {

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> anyhow::Result<()> {
        self.editor.draw(f, area, focused)
    }
}

impl Component for CommandEditorComponent {

    fn event(&mut self, key: &[Key]) -> anyhow::Result<EventState> {
        self.editor.event(key)
    }
}