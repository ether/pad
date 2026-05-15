use clap::Parser;
use pad::cli::{Args, Mode};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.mode() {
        Mode::Untitled => println!("untitled buffer"),
        Mode::OpenFile(p) => println!("open: {}", p.display()),
        Mode::Recover => println!("recover mode"),
    }
    Ok(())
}
