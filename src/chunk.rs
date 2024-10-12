/// 3.1 PNG file format
pub const FILE_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

pub fn valid_type_code(byte: u8) -> bool {
    matches!(byte, 65..=90 | 97..=122)
}
