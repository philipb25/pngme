use std::error;
use std::fmt::Display;
use std::io::{self, BufReader, Read};

use crate::chunk_type::{ChunkType, TryFromChunkTypeError};
use crate::Result;

pub struct Chunk {
    len: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let len = u32::try_from(data.len()).unwrap();
        let crc = calculate_crc(&chunk_type, &data);
        Self {
            len,
            chunk_type,
            data,
            crc,
        }
    }

    pub fn length(&self) -> u32 {
        self.len
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..] // or `self.data.as_slice`
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String> {
        Ok(String::from_utf8(self.data.clone())?)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.len
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.as_slice().iter())
            .chain(&self.data)
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("len", &self.len)
            .field("chunk_type", &self.chunk_type)
            .field("data", &self.data.len())
            .field("crc", &self.crc)
            .finish()
    }
}

fn calculate_crc(chunk_type: &ChunkType, data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(chunk_type.as_slice());
    hasher.update(data);
    hasher.finalize()
}

#[derive(Debug)]
#[non_exhaustive]
pub struct TryFromBytesError {
    kind: TryFromBytesErrorKind,
}

impl TryFromBytesError {
    fn new(kind: TryFromBytesErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for TryFromBytesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cannot convert to Chunk")
    }
}

impl error::Error for TryFromBytesError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.kind)
    }
}

#[derive(Debug)]
#[non_exhaustive]
enum TryFromBytesErrorKind {
    ChunkType(TryFromChunkTypeError),
    CorruptCrc {
        calculated: u32,
        expected: u32,
    },
    #[non_exhaustive]
    NotEnoughBytes(io::Error),
}

impl Display for TryFromBytesErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TryFromBytesErrorKind::ChunkType(err) => Display::fmt(err, f),
            TryFromBytesErrorKind::CorruptCrc {
                calculated,
                expected,
            } => write!(f, "invalid crc: expected: {expected}, got {calculated}"),
            TryFromBytesErrorKind::NotEnoughBytes(err) => Display::fmt(err, f),
        }
    }
}

impl error::Error for TryFromBytesErrorKind {}

impl From<TryFromChunkTypeError> for TryFromBytesError {
    fn from(err: TryFromChunkTypeError) -> Self {
        Self::new(TryFromBytesErrorKind::ChunkType(err))
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = TryFromBytesError;

    fn try_from(bytes: &[u8]) -> std::result::Result<Self, Self::Error> {
        let mut bytes = BufReader::new(bytes);
        let mut buffer = [0u8; 4];
        bytes
            .read_exact(&mut buffer)
            .map_err(|e| TryFromBytesError::new(TryFromBytesErrorKind::NotEnoughBytes(e)))?;
        let len = u32::from_be_bytes(buffer);

        bytes
            .read_exact(&mut buffer)
            .map_err(|e| TryFromBytesError::new(TryFromBytesErrorKind::NotEnoughBytes(e)))?;
        let chunk_type = ChunkType::try_from(buffer)?;

        if len == 0 {
            let crc = calculate_crc(&chunk_type, &[]);
            return Ok(Self {
                len,
                chunk_type,
                data: Vec::new(),
                crc,
            });
        }

        let mut data = vec![0u8; len as usize];
        bytes
            .read_exact(&mut data)
            .map_err(|e| TryFromBytesError::new(TryFromBytesErrorKind::NotEnoughBytes(e)))?;

        let crc = calculate_crc(&chunk_type, &data[..]);

        bytes
            .read_exact(&mut buffer)
            .map_err(|e| TryFromBytesError::new(TryFromBytesErrorKind::NotEnoughBytes(e)))?;
        let crc_given = u32::from_be_bytes(buffer);
        if crc != crc_given {
            return Err(TryFromBytesError::new(TryFromBytesErrorKind::CorruptCrc {
                calculated: crc,
                expected: crc_given,
            }));
        }
        Ok(Self {
            len,
            chunk_type,
            data,
            crc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
