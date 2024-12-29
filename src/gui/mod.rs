use std::{fs::File, io::Read};

use anyhow::Result;
use dirs::config_dir;
use iced::{
    border::Radius,
    widget::{button, text},
    Length, Size, Task, Theme,
};
use iced_aw::{menu::Item, menu_bar, menu_items, Menu};
use serde::{Deserialize, Serialize};

use crate::core::Gba;

#[derive(Default, Serialize, Deserialize)]
struct Settings {
    bios_path: Option<String>,
}

impl Settings {
    pub fn load() -> Result<Self> {
        if let Some(app_data) = config_dir() {
            let app_dir = app_data.join("rgba");
            let settings_file = app_dir.join("settings.json");

            if !app_dir.exists() {
                std::fs::create_dir_all(&app_dir)?;
            }

            if let Ok(mut file) = File::open(&settings_file) {
                let mut buf = Vec::new();
                let length = file.read_to_end(&mut buf)?;
                return Ok(serde_json::from_slice(&buf[..length])?);
            }
        }

        Ok(Settings::default())
    }
}

#[derive(Debug, Clone)]
enum Message {
    Exit,
}

pub struct MainWindow {
    gba: Gba,
    settings: Settings,
}

impl Default for MainWindow {
    fn default() -> Self {
        let mut gba = Gba::new();

        let settings = if let Ok(settings) = Settings::load() {
            settings
        } else {
            println!("Failed to load settings, using default settings");
            Settings::default()
        };

        if let Some(bios_path) = &settings.bios_path {
            // TODO: This shouldn't load right away. Only when the emulation is started
            gba.set_bios(bios_path).unwrap();
        }

        Self {
            gba,
            settings,
        }
    }
}

impl MainWindow {
    pub fn show(&self) -> iced::Result {
        iced::application("RGBA Emulator", Self::update, Self::view)
            .window_size(Size::new(240.0 * 2.0, 160.0 * 2.0))
            .run()
    }

    fn view(&self) -> iced::Element<Message> {
        let menu_template = |items| Menu::new(items).max_width(180.0).offset(5.0).spacing(0.0);

        #[rustfmt::skip]
        let top_bar = menu_bar!(
            (base_button("File"), {
                menu_template(menu_items!(
                    (menu_button("Open ROM"))
                    (menu_button("Exit").on_press(Message::Exit))
                ))
                .width(160.0)
            })
        );
        top_bar.into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Exit => iced::exit(),
        }
    }
}

fn menu_button_style(theme: &Theme, status: button::Status) -> iced::widget::button::Style {
    let mut style = button::primary(theme, status);
    style.border.width = 0.0;
    style.border.radius = Radius::new(0.0);
    style
}

fn base_button(label: &str) -> button::Button<Message, iced::Theme, iced::Renderer> {
    button(text(label)).style(menu_button_style)
}

fn menu_button(label: &str) -> button::Button<Message, iced::Theme, iced::Renderer> {
    base_button(label).width(Length::Fill)
}
