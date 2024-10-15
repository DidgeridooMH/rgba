mod core;
use crate::core::Gba;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    bios: String,
    #[arg(short, long)]
    cycles: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut gba = Gba::new(&args.bios)?;
    gba.emulate(args.cycles)?;

    Ok(())
}
