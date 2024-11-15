use std::convert::TryFrom;
use std::error::Error;
use std::fmt::Display;
use std::str::{self, FromStr};

#[derive(Debug, PartialEq, Eq)]
pub struct ChunkType {
    bytes: [u8; 4],
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..]).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }

    pub fn is_critical(&self) -> bool {
        (self.bytes[0] & 32u8) != 32
    }

    pub fn is_public(&self) -> bool {
        (self.bytes[1] & 32u8) != 32
    }

    pub fn is_reserved_bit_valid(&self) -> bool {
        (self.bytes[2] & 32u8) != 32
    }

    pub fn is_safe_to_copy(&self) -> bool {
        (self.bytes[3] & 32u8) == 32
    }
}

#[derive(Debug)]
pub struct TryFromChunkTypeError {
    index: usize,
    value: u8,
}

impl Display for TryFromChunkTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid byte `{}` at index {}", self.value, self.index)
    }
}

impl Error for TryFromChunkTypeError {}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = TryFromChunkTypeError;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        for (i, val) in value.iter().enumerate() {
            if !matches!(val, 65..=90 | 97..=122) {
                return Err(Self::Error {
                    index: i,
                    value: *val,
                });
            }
        }
        Ok(Self { bytes: value })
    }
}

#[derive(Debug)]
pub enum ParseChunkTypeError {
    InvalidLength,
    NotInRange(char),
}

impl Display for ParseChunkTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseChunkTypeError::InvalidLength => f.write_str("length needs to be 4"),
            ParseChunkTypeError::NotInRange(value) => write!(
                f,
                "invalid value `{}`, acceptable range 'A-Z and a-z'",
                value
            ),
        }
    }
}

impl Error for ParseChunkTypeError {}

impl FromStr for ChunkType {
    type Err = ParseChunkTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 4 {
            return Err(ParseChunkTypeError::InvalidLength);
        }
        let mut buf = [0; 4];
        buf.copy_from_slice(bytes);
        Self::try_from(buf).map_err(|e| ParseChunkTypeError::NotInRange(char::from(e.value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_parse_method() -> Result<(), ParseChunkTypeError> {
        let _chunk = "RuSt".parse::<ChunkType>()?;
        Ok(())
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid(), "chunk invalid");

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err(), "chunk NOT an error when it should be");
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
