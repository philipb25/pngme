use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    /// Encode message into a png file.
    Encode(EncodeArgs),
    /// Decode a message out of a png file.
    Decode(DecodeArgs),
    /// Remove a chunk from png file.
    Remove(RemoveArgs),
    /// Print png file.
    Print(PrintArgs),
}

#[derive(Debug, clap::Args)]
pub struct EncodeArgs {
    pub png_file: PathBuf,
    pub chunk_type: String,
    pub message: String,
}

#[derive(Debug, clap::Args)]
pub struct DecodeArgs {
    pub png_file: PathBuf,
    pub chunk_type: String,
}

#[derive(Debug, clap::Args)]
pub struct RemoveArgs {
    pub png_file: PathBuf,
    pub chunk_type: String,
}

#[derive(Debug, clap::Args)]
pub struct PrintArgs {
    pub png_file: PathBuf,
}
