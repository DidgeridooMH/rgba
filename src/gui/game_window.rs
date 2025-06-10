use std::any::Any;

use iced::{
    widget::{button, center, column, container, text},
    Element,
};
use iced_aw::{menu::Item, menu_bar, menu_items, Menu};

use super::{Message, Window, WindowClass};

pub struct GameWindow;

impl GameWindow {
    pub fn new() -> Self {
        Self {}
    }

    fn menu_bar(&self) -> Element<Message> {
        let menu_template = |items| Menu::new(items).max_width(180.0).offset(5.0).spacing(0.0);

        #[rustfmt::skip]
        let bar = menu_bar!(
            (button(text("File")), {
                menu_template(menu_items!(
                    (menu_button("Open ROM", Message::OpenRom))
                    (menu_button("Close", Message::Exit))
                ))
                .width(100.0)
            })
            (button(text("Tools")), {
                menu_template(menu_items!(
                    (button(text("Debugger")).on_press(Message::OpenWindow(WindowClass::Debugger)))
                ))
                .width(100.0)
            })
        )
        .into();

        bar
    }
}

fn menu_button(label: &str, msg: Message) -> Element<Message> {
    button(text(label).align_y(iced::Alignment::Center))
        .padding([4, 8])
        .style(iced::widget::button::primary)
        .width(iced::Length::Fill)
        .on_press(msg)
        .into()
}

impl Window for GameWindow {
    fn title(&self) -> String {
        "RGBA Emulator".to_string()
    }

    fn view(&self) -> iced::Element<Message> {
        container(column![
            self.menu_bar(),
            center(text("Game rendering not implemented yet...")),
        ])
        .into()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
