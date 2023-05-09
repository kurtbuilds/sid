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
    23, 24, 25, 26, 27, 28, 29, 30, 31, 255, 0, 255, 1,
    2, 3, 4, 5, 6, 7, 8, 255, 9, 10, 11, 12, 13,
    14, 15, 16, 255, 17, 18, 255, 19, 20, 255, 21, 22
];

pub const SHORT_LENGTH: usize = 22;

// really in range 0..32 (or 255 if error)
#[inline]
fn lookup(value: u8) -> u8 {
    let value = value % LOOKUP_TABLE_LENGTH;
    REVERSE_ALPHABET[value as usize]
}

pub fn base32_decode(input: &str) -> Result<[u8; 16], DecodeError> {
    if input.len() != 27 {
        return Err(DecodeError::InvalidLength);
    }
    let input: [u8; 27] = input.as_bytes().try_into().unwrap();
    if !input[22] == b'_' {
        return Err(DecodeError::NoSeparator);
    }

    // build intermediate map. u8 data type, but each only contains 5 bits.
    let mut intermediate = [0u8; 26];
    for i in 0..22 {
        intermediate[i] = lookup(input[i]);
    }
    for i in 23..27 {
        intermediate[i - 1] = lookup(input[i]);
    }

    // bit hacks to check if any invalid with minimal branching.
    let mut combined = 0u8;
    for &c in &intermediate {
        combined |= c;
    }
    let has_invalid = (combined & !(combined.saturating_sub(255))) == 255;
    if has_invalid {
        let mut idx255 = intermediate.iter().position(|&c| c == 255).unwrap();
        if idx255 >= 22 {
            idx255 += 1;
        }
        let c = input[idx255];
        return Err(DecodeError::InvalidCharacter(c as char));
    }

    let mut result = [0u8; 16];
    // now we can do the actual decoding.
    for i in 0..3 {
        let j = i * 8;
        let k = i * 5;
        let d0 = intermediate[j];
        let d1 = intermediate[j + 1];
        let d2 = intermediate[j + 2];
        let d3 = intermediate[j + 3];
        let d4 = intermediate[j + 4];
        let d5 = intermediate[j + 5];
        let d6 = intermediate[j + 6];
        let d7 = intermediate[j + 7];
        let d8 = intermediate[j + 8];

        result[k] = d0 << 5 | d1;
        result[k + 1] = d2 << 3 | (d3 >> 2);
        result[k + 2] = d3 << 6 | (d4 << 1) | (d5 >> 4);
        result[k + 3] = d5 << 4 | (d6 >> 1);
        result[k + 4] = d6 << 7 | (d7 << 2) | (d8 >> 3);
    }

    result[15] = intermediate[24] << 5 | intermediate[25];

    Ok(result)
}

#[inline]
fn alphabet(i: u8) -> char {
    // unsafe {
    //     *ALPHABET.get_unchecked(i & 0x1F as usize)
    // }
    ALPHABET[(i & 0x1F) as usize]
}

pub fn base32_encode(data: [u8; 16]) -> String {
    let mut encoded = String::with_capacity(27);
    // encoded0 skips 2 bits and takes top 3 bits from data0
    encoded.push(alphabet(data[0] >> 5));
    // encoded1 takes bottom 5 bits from data0
    encoded.push(alphabet(data[0]));
    // encoded2 takes top 5 bits from data1
    encoded.push(alphabet(data[1] >> 3));
    // encoded3 takes bottom 3 bits from data1 and top 2 bits from data2
    encoded.push(alphabet(data[1] << 2 | data[2] >> 6));
    // encoded4 takes bits 3-7 from data2
    encoded.push(alphabet(data[2] >> 1));
    // encoded5 takes bottom 1 bit from data2 and top 4 bits from data3
    encoded.push(alphabet(data[2] << 4 | data[3] >> 4));
    // encoded6 takes bottom 4 bits from data3 and top 1 bit from data4
    encoded.push(alphabet(data[3] << 1 | data[4] >> 7));
    // encoded7 takes bits 2-6 from data4
    encoded.push(alphabet(data[4] >> 2));
    // encoded8 takes bottom 2 bits from data4 and top 3 bits from data5
    encoded.push(alphabet(data[4] << 3 | data[5] >> 5));
    // encoded9 takes bottom 5 bits from data5
    encoded.push(alphabet(data[5]));
    // encoded10 takes top 5 bits from data6
    encoded.push(alphabet(data[6] >> 3));
    // encoded11 takes bottom 3 bits from data6 and top 2 bits from data7
    encoded.push(alphabet(data[6] << 2 | data[7] >> 6));
    // encoded12 takes bits 3-7 from data7
    encoded.push(alphabet(data[7] >> 1));
    // encoded13 takes bottom 1 bit from data7 and top 4 bits from data8
    encoded.push(alphabet(data[7] << 4 | data[8] >> 4));
    // encoded14 takes bottom 4 bits from data8 and top 1 bit from data9
    encoded.push(alphabet(data[8] << 1 | data[9] >> 7));
    // encoded15 takes bits 2-6 from data9
    encoded.push(alphabet(data[9] >> 2));
    // encoded16 takes bottom 2 bits from data9 and top 3 bits from data10
    encoded.push(alphabet(data[9] << 3 | data[10] >> 5));
    // encoded17 takes bottom 5 bits from data10
    encoded.push(alphabet(data[10]));
    // encoded18 takes top 5 bits from data11
    encoded.push(alphabet(data[11] >> 3));
    // encoded19 takes bottom 3 bits from data11 and top 2 bits from data12
    encoded.push(alphabet(data[11] << 2 | data[12] >> 6));
    // encoded20 takes bits 3-7 from data12
    encoded.push(alphabet(data[12] >> 1));
    // encoded21 takes bottom 1 bit from data12 and top 4 bits from data13
    encoded.push(alphabet(data[12] << 4 | data[13] >> 4));
    encoded.push('_');
    // encoded22 takes bottom 4 bits from data13 and top 1 bit from data14
    encoded.push(alphabet(data[13] << 1 | data[14] >> 7));
    // encoded23 takes bits 2-6 from data14
    encoded.push(alphabet(data[14] >> 2));
    // encoded24 takes bottom 2 bits from data14 and top 3 bits from data15
    encoded.push(alphabet(data[14] << 3 | data[15] >> 5));
    // encoded25 takes bottom 5 bits from data15
    encoded.push(alphabet(data[15]));
    encoded
}

#[cfg(test)]
mod test {
    use super::*;

    // iterating 2**128 times isn't a great idea
    // #[test]
    // fn test_ridiculous() {
    //     let mut s = [0u8; 16];
    //     let mut last_enc = "".to_string();
    //     loop {
    //         // Increment the counter
    //         for i in (0..16).rev() {
    //             if s[i] < 255 {
    //                 s[i] += 1;
    //                 break;
    //             } else {
    //                 if i < 15 {
    //                     println!("pos {i} reset to 0");
    //                 }
    //                 s[i] = 0;
    //             }
    //         }
    //
    //         let enc = base32_encode(s);
    //         assert_eq!(enc > last_enc, true);
    //         let dec = base32_decode(&enc).unwrap();
    //         let last_enc = enc;
    //         assert_eq!(dec, s);
    //
    //         // Check if the counter reached the maximum value
    //         if s.iter().all(|&x| x == 255) {
    //             break;
    //         }
    //     }
    // }

    #[test]
    fn test_rand() {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        for _ in 0..3000000 {
            let mut s = [0u8; 16];
            rng.fill_bytes(&mut s);
            let mut t = [0u8; 16];
            rng.fill_bytes(&mut t);

            // test s round trips
            let s_enc = base32_encode(s);
            let dec = base32_decode(&s_enc).unwrap();
            assert_eq!(dec, s);

            // test t round trips
            let t_enc = base32_encode(t);
            let dec = base32_decode(&t_enc).unwrap();
            assert_eq!(dec, t);

            if s < t {
                assert_eq!(s_enc < t_enc, true);
            } else {
                assert_eq!(s_enc > t_enc, true);
            }
        }
    }
}