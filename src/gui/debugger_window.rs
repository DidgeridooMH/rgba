use iced::widget::{center, text};

use super::{Message, Window};

pub struct DebuggerWindow;

impl DebuggerWindow {
    pub fn new() -> Self {
        Self {}
    }
}

impl Window for DebuggerWindow {
    fn title(&self) -> String {
        "Debugger".to_string()
    }

    fn view(&self) -> iced::Element<Message> {
        center(text("Debugger Window")).into()
    }
}
