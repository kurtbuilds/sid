mod error;

pub use error::DecodeError;

static ALPHABET: [char; 32] = [
    '0', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd',
    'e', 'f', 'g', 'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't',
    'u', 'v', 'w', 'x', 'y', 'z',
];

const LOOKUP_TABLE_LENGTH: u8 = 38;
// generated with some trial and error in the perfect_hash.py file.
static REVERSE_ALPHABET: [u8; LOOKUP_TABLE_LENGTH as usize] = [
    23, 24, 25, 26, 27, 28, 29, 30, 31, 0, 0, 0, 1, 2, 3, 4, 5,
    6, 7, 8, 0, 9, 10, 11, 12, 13, 14, 15, 16, 0, 17, 18, 0, 19,
    20, 0, 21, 22
];

pub const SHORT_LENGTH: usize = 22;

pub fn base32_decode(input: &str) -> Result<[u8; 16], DecodeError> {
    if input.len() != 27 || input.as_bytes()[SHORT_LENGTH] != b'_' {
        return Err(DecodeError("Malformed input. If input contains a label, strip it first.".to_string()));
    }

    let mut decoded: [u8; 16] = [0; 16];
    let mut buffer = [0u8; 8];
    let mut decoded_idx = 0;
    let mut string_idx = 0;

    for &c in input.as_bytes().iter() {
        if c == 95 {
            continue;
        }
        let reduced = c % LOOKUP_TABLE_LENGTH;
        let value = REVERSE_ALPHABET[reduced as usize];
        let buffer_idx = string_idx % 8;

        string_idx += 1;
        buffer[buffer_idx] = value;
        if buffer_idx == 7 {
            decoded[decoded_idx] = (buffer[0] << 3) | (buffer[1] >> 2);
            decoded[decoded_idx + 1] = (buffer[1] << 6) | (buffer[2] << 1) | (buffer[3] >> 4);
            decoded[decoded_idx + 2] = (buffer[3] << 4) | (buffer[4] >> 1);
            decoded[decoded_idx + 3] = (buffer[4] << 7) | (buffer[5] << 2) | (buffer[6] >> 3);
            decoded[decoded_idx + 4] = (buffer[6] << 5) | buffer[7];
            decoded_idx += 5;
        }
    }
    decoded[15] = (buffer[0] << 3) | (buffer[1] >> 2);
    Ok(decoded)
}

pub fn base32_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut buffer = 0u64;
    let mut bits_in_buffer = 0usize;

    for byte in data {
        buffer = (buffer << 8) | (*byte as u64);
        bits_in_buffer += 8;

        while bits_in_buffer >= 5 {
            bits_in_buffer -= 5;
            let index = ((buffer >> bits_in_buffer) & 0x1F) as usize;
            result.push(ALPHABET[index]);
        }
    }

    if bits_in_buffer > 0 {
        let index = ((buffer << (5 - bits_in_buffer)) & 0x1F) as usize;
        result.push(ALPHABET[index]);
    }

    result
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU64;
    use super::*;
}