use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub use error::DecodeError;
pub use label::Label;

mod error;
mod label;

static ALPHABET: [char; 32] = [
    '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j', 'k', 'm', 'n',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '2', '3',
    '4', '5', '6', '7', '8', '9',
];

const LOOKUP_TABLE_LENGTH: u8 = 38;
// generated with some trial and error in the perfect_hash.py file.
static REVERSE_ALPHABET: [u8; LOOKUP_TABLE_LENGTH as usize] = [
    15, 16, 17, 18, 19, 20, 21, 22, 23, 0, 0, 0, 24, 25, 26, 27, 28,
    29, 30, 31, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 9, 10, 0, 11,
    12, 0, 13, 14
];

const SHORT_LENGTH: usize = 22;

fn base32_decode(input: &str) -> Result<[u8; 16], DecodeError> {
    if input.len() != 27 || input.as_bytes()[22] != b'_' {
        return Err(DecodeError("Malformed input.".to_string()));
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

fn base32_encode(data: &[u8]) -> String {
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

#[cfg(all(feature = "rand", any(feature = "chrono", feature = "time")))]
fn unix_epoch_ms() -> u64 {
    #[cfg(feature = "time")]
    {
        let now = time::OffsetDateTime::now_utc();

        now.unix_timestamp() as u64 * 1_000 + now.millisecond() as u64
    }
    #[cfg(all(feature = "chrono", not(feature = "time")))]
    {
        let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();

        now.timestamp_millis() as u64
    }
}

pub struct NoLabel;

#[derive(PartialOrd, PartialEq, Eq, Ord, Hash, Copy)]
pub struct Oid<T = NoLabel> {
    data: [u8; 16],
    marker: std::marker::PhantomData<T>,
}

impl<T> Clone for Oid<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            marker: Default::default(),
        }
    }
}

pub fn new_oid() -> Oid {
    NoLabel::oid()
}

impl<T: Label> Oid<T> {
    #[cfg(feature = "rand")]
    pub fn from_timestamp_with_rng<R>(timestamp: u64, rng: &mut R) -> Self
        where
            R: rand::Rng,
    {
        if (timestamp & 0xFFFF_0000_0000_0000) != 0 {
            panic!("oid does not support timestamps after +10889-08-02T05:31:50.655Z");
        }
        let high = timestamp << 16 | u64::from(rng.gen::<u16>());
        let low = rng.gen::<u64>();
        let high = high.to_le_bytes();
        let low = low.to_le_bytes();

        let mut data: [u8; 16] = [0; 16];
        data[..8].copy_from_slice(&high);
        data[8..].copy_from_slice(&low);

        Self {
            data,
            marker: Default::default(),
        }
    }

    pub fn short(&self) -> String {
        let encoded = base32_encode(&self.data);
        format!("{}{}", T::label(), &encoded[SHORT_LENGTH..])
    }

    pub fn is_null(&self) -> bool {
        self.data.iter().all(|&b| b == 0)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[cfg(feature = "uuid")]
    pub fn uuid(&self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.data)
    }
}

#[cfg(feature = "uuid")]
impl<T> Into<uuid::Uuid> for Oid<T> {
    fn into(self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.data)
    }
}

#[cfg(feature = "uuid")]
impl<T: Label> From<uuid::Uuid> for Oid<T> {
    fn from(value: uuid::Uuid) -> Self {
        let bytes = value.as_ref();
        let mut data: [u8; 16] = [0; 16];
        data.copy_from_slice(bytes);
        T::from_bytes(data)
    }
}

impl<T: Label> FromStr for Oid<T> {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = base32_decode(s)?;
        Ok(T::from_bytes(data))
    }
}

impl<T: Label> Debug for Oid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let encoded = base32_encode(&self.data);
        write!(f, "{}{}_{}",
               T::label(),
               &encoded[..SHORT_LENGTH],
               &encoded[SHORT_LENGTH..],
        )
    }
}


#[macro_export]
macro_rules! oid {
    ($value:ident) => {
        oid!(stringify!($value))
    };
    ($value:expr) => {{
        let value = $value;
        let count = value.matches('_').count();
        match count {
            1 => Oid::from_str(value).unwrap(),
            2 => {
                let value = value.splitn(2, '_').nth(1).unwrap();
                Oid::<_>::from_str(value).unwrap()
            },
            _ => panic!("oid must have 1 or 2 underscores."),
        }
    }};
}

impl<T: Label> Display for Oid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use label::Label;

    use super::*;

    label!(Team, "team");

    #[test]
    fn it_works() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let oid = Team::from_bytes(bytes);
        println!("{}", oid.short());
        println!("{}", oid);
        assert_eq!(oid.to_string(), "team_0da0fa0e02cssbhkanf04c_srb0");
        assert_eq!(oid.short(), "team_srb0");
    }

    #[test]
    fn test_null() {
        let oid = Team::null();
        println!("{}", oid.short());
        println!("{}", oid);
        assert_eq!(oid.to_string(), "team_0000000000000000000000_0000");
        assert_eq!(oid.short(), "team_0000");
        let oid = NoLabel::null();
        assert_eq!(oid.to_string(), "0000000000000000000000_0000");
    }

    #[test]
    #[cfg(feature = "uuid")]
    fn test_uuid() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let oid = Team::from_bytes(bytes);
        let uuid: uuid::Uuid = oid.clone().into();
        assert_eq!(uuid.to_string(), "01020304-0506-0708-090a-0b0c0d0e0f10");
        let uuid2 = oid.uuid();
        assert_eq!(uuid, uuid2);
    }

    #[test]
    fn test_macro() {
        let oid: Oid<Team> = oid!("team_0000000000000000000000_0000");
        assert!(oid.is_null(), "{}", oid);
        let oid: Oid<Team> = oid!(team_0000000000000000000000_0000);
        assert!(oid.is_null(), "{}", oid);
        let oid: Oid<Team> = oid!(team_0da0fa0e02cssbhkanf04c_srb0);
        dbg!(oid.data());
        assert_eq!(oid.to_string(), "team_0da0fa0e02cssbhkanf04c_srb0");
    }
}
