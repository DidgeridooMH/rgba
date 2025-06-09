use std::{collections::BTreeMap, fs::File, io::Read};

use anyhow::Result;
use dirs::config_dir;
use iced::{widget::horizontal_space, window, Element, Size, Task, Vector};
use serde::{Deserialize, Serialize};

mod game_window;
use game_window::GameWindow;

mod debugger_window;
use debugger_window::DebuggerWindow;

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
pub enum Message {
    OpenWindow(WindowClass),
    WindowOpened((window::Id, WindowClass)),
    WindowClosed(window::Id),
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WindowClass {
    Game,
    Debugger,
}

trait Window {
    fn title(&self) -> String;
    fn view(&self) -> Element<Message>;
}

pub struct Application {
    windows: BTreeMap<window::Id, Box<dyn Window>>,
    game_window: Option<window::Id>,
    debugger_window: Option<window::Id>,
}

/*impl Default for MainWindow {
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

        Self { gba, settings }
    }
}*/

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        let (_id, open) = window::open(window::Settings {
            size: Size::new(240.0 * 2.0, 160.0 * 2.0),
            ..Default::default()
        });

        (
            Self {
                windows: BTreeMap::new(),
                game_window: None,
                debugger_window: None,
            },
            open.map(|id| Message::WindowOpened((id, WindowClass::Game))),
        )
    }

    pub fn title(&self, window: window::Id) -> String {
        self.windows
            .get(&window)
            .map_or("Unknown window".into(), |w| w.title())
    }

    pub fn view(&self, window_id: window::Id) -> iced::Element<Message> {
        self.windows
            .get(&window_id)
            .map_or(horizontal_space().into(), |window| window.view())

        //let id = Task::perform(window::get_latest());

        //if id {
        //let menu_template = |items| Menu::new(items).max_width(180.0).offset(5.0).spacing(0.0);

        //#[rustfmt::skip]
        //    let top_bar = menu_bar!(
        //        (base_button("File"), {
        //            menu_template(menu_items!(
        //                (menu_button("Open ROM"))
        //                (menu_button("Exit").on_press(Message::Exit))
        //            ))
        //            .width(160.0)
        //        })
        //        (base_button("Tools"), {
        //            menu_template(menu_items!(
        //                (menu_button("Debugger").on_press(Message::OpenDebugger))
        //            ))
        //            .width(160.0)
        //        })
        //    );
        //top_bar.into()
        //} else {
        //   iced::Element::new(text("Debugger"))
        //}
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenWindow(window_class) => {
                let Some(last_window) = self.windows.keys().last() else {
                    return Task::none();
                };

                match window_class {
                    WindowClass::Game if self.game_window.is_some() => return Task::none(),
                    WindowClass::Debugger if self.debugger_window.is_some() => return Task::none(),
                    _ => {}
                }

                window::get_position(*last_window)
                    .then(|last_position| {
                        let position =
                            last_position.map_or(window::Position::Default, |last_position| {
                                window::Position::Specific(last_position + Vector::new(20.0, 20.0))
                            });

                        let (_id, open) = window::open(window::Settings {
                            position,
                            ..window::Settings::default()
                        });

                        open
                    })
                    .map(move |id| Message::WindowOpened((id, window_class)))
            }
            Message::WindowOpened((id, window_class)) => {
                match window_class {
                    WindowClass::Game => {
                        if self.game_window.is_none() {
                            self.game_window = Some(id);
                            self.windows.insert(id, Box::new(GameWindow::new()));
                        }
                    }
                    WindowClass::Debugger => {
                        if self.debugger_window.is_none() {
                            self.debugger_window = Some(id);
                            self.windows.insert(id, Box::new(DebuggerWindow::new()));
                        }
                    }
                }
                Task::none()
            }
            Message::WindowClosed(id) => {
                self.windows.remove(&id);
                if self.windows.is_empty() || self.game_window == Some(id) {
                    return iced::exit();
                }

                if self.game_window == Some(id) {
                    self.game_window = None;
                } else if self.debugger_window == Some(id) {
                    self.debugger_window = None;
                }

                Task::none()
            }
            Message::Exit => iced::exit()
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}
