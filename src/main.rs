mod core;
use crate::core::Gba;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    bios: String,
}

fn main() {
    let args = Args::parse();

    let gba = Gba::new(&args.bios);
}
