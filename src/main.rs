mod core;
mod gui;

use anyhow::Result;
use clap::Parser;
use gui::Application;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    cycles: Option<usize>,
}


fn main() -> Result<()> {
    let _args = Args::parse();

    iced::daemon(Application::title, Application::update, Application::view)
        .subscription(Application::subscription)
        .run_with(Application::new)?;

    //    let mut gba = Gba::new(&args.bios)?;
    //    gba.emulate(args.cycles)?;

    Ok(())
}
