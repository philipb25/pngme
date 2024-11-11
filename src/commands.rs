use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::io;
use std::path::Path;

use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use crate::png::{self, Png};
use crate::AnyError;

pub fn encode(path: &Path, chunk_type: &str, message: &str) -> Result<(), AnyError> {
    let mut png = read_png(path)?;
    let chunk = Chunk::new(
        chunk_type.parse::<ChunkType>()?,
        message.as_bytes().to_owned(),
    );
    png.append_chunk(chunk);
    fs::write(path, png.as_bytes())?;
    Ok(())
}

pub fn decode(path: &Path, chunk_type: &str) -> Result<(), AnyError> {
    let png = read_png(path)?;
    match png.chunk_by_type(chunk_type) {
        Some(chunk) => {
            println!("[i] Chunk found: {chunk:#}");
            println!("[i] Secret message: {}.", chunk.data_as_string()?);
        }
        None => println!("Chunk type: `{chunk_type}` not found."),
    }
    Ok(())
}

pub fn remove(path: &Path, chunk_type: &str) -> Result<(), AnyError> {
    let mut png = read_png(path)?;
    png.remove_first_chunk(chunk_type)?;
    fs::write(path, png.as_bytes())?;
    Ok(())
}

pub fn print(path: &Path) -> Result<(), ReadPngError> {
    let png = read_png(path)?;
    for chunk in png.chunks() {
        println!("{chunk:#}")
    }
    Ok(())
}

fn read_png(path: &Path) -> Result<Png, ReadPngError> {
    let contents = fs::read(path)?;
    let png = Png::try_from(&contents[..])?;

    Ok(png)
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ReadPngError {
    Png(png::TryFromSliceError),
    Io(io::Error),
}

impl Display for ReadPngError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadPngError::Png(err) => Display::fmt(err, f),
            ReadPngError::Io(err) => Display::fmt(err, f),
        }
    }
}

impl Error for ReadPngError {}

impl From<png::TryFromSliceError> for ReadPngError {
    fn from(err: png::TryFromSliceError) -> Self {
        ReadPngError::Png(err)
    }
}

impl From<io::Error> for ReadPngError {
    fn from(err: io::Error) -> Self {
        ReadPngError::Io(err)
    }
}
