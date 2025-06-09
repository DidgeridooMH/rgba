use iced::widget::{button, center, column, container, text};

use super::{Message, Window, WindowClass};

pub struct GameWindow;

impl GameWindow {
    pub fn new() -> Self {
        Self {}
    }
}

impl Window for GameWindow {
    fn title(&self) -> String {
        "RGBA Emulator".to_string()
    }

    fn view(&self) -> iced::Element<Message> {
        container(column![
            center(text("Game Window")),
            button(text("Open Debugger"))
                .on_press(Message::OpenWindow(WindowClass::Debugger))
        ]).into()
    }
}
