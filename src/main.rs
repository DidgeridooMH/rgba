mod core;
mod gui;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    cycles: Option<usize>,
}


fn main() -> Result<()> {
    let _args = Args::parse();

    let main_window = gui::MainWindow::default();
    main_window.show()?;

    //    let mut gba = Gba::new(&args.bios)?;
    //    gba.emulate(args.cycles)?;

    Ok(())
}
