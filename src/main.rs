pub mod args;
pub mod chunk;
pub mod chunk_type;
pub mod commands;
pub mod png;

use clap::Parser;

use crate::args::Args;

pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, AnyError>;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    match args.command {
        args::Commands::Encode(args) => {
            commands::encode(&args.png_file, &args.chunk_type, &args.message)?
        }
        args::Commands::Decode(args) => commands::decode(&args.png_file, &args.chunk_type)?,
        args::Commands::Remove(args) => commands::remove(&args.png_file, &args.chunk_type)?,
        args::Commands::Print(args) => commands::print(&args.png_file)?,
    }

    Ok(())
}
