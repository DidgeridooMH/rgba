mod core;
mod gui;

use anyhow::Result;
use gui::Application;

fn main() -> Result<()> {
    iced::daemon(Application::title, Application::update, Application::view)
        .subscription(Application::subscription)
        .run_with(Application::new)?;

    Ok(())
}
