mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let bytes = [
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 20, 70, 114, 83, 116, 73, 32, 97, 109, 32, 116,
        104, 101, 32, 102, 105, 114, 115, 116, 32, 99, 104, 117, 110, 107, 54, 220, 183, 239, 0, 0,
        0, 18, 109, 105, 68, 108, 73, 32, 97, 109, 32, 97, 110, 111, 116, 104, 101, 114, 32, 99,
        104, 117, 110, 107, 199, 116, 207, 18, 0, 0, 0, 19, 76, 65, 83, 116, 73, 32, 97, 109, 32,
        116, 104, 101, 32, 108, 97, 115, 116, 32, 99, 104, 117, 110, 107, 18, 176, 96, 100,
    ];
    let image = png::Png::try_from(&bytes[..]).inspect_err(|e| eprintln!("{e:#?}"))?;
    for chunk in image.chunks() {
        eprintln!("{chunk}");
    }
    assert_eq!(&bytes[..], &image.as_bytes()[..]);
    Ok(())
}
