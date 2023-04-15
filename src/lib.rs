use std::fmt::{Debug, Display};

static ALPHABET: &str = "0abcdefghjkmnpqrstuvwxyz23456789";

const SHORT_LENGTH: usize = 22;

#[derive(Clone)]
pub struct Test<T> {
    _marker: T,
}

fn base32_encode(data: &[u8], alphabet: &str) -> String {
    let mut result = String::new();
    let mut buffer = 0u64;
    let mut bits_in_buffer = 0usize;

    let alphabet_chars: Vec<char> = alphabet.chars().collect();

    for byte in data {
        buffer = (buffer << 8) | (*byte as u64);
        bits_in_buffer += 8;

        while bits_in_buffer >= 5 {
            bits_in_buffer -= 5;
            let index = ((buffer >> bits_in_buffer) & 0x1F) as usize;
            result.push(alphabet_chars[index]);
        }
    }

    if bits_in_buffer > 0 {
        let index = ((buffer << (5 - bits_in_buffer)) & 0x1F) as usize;
        result.push(alphabet_chars[index]);
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

impl Label for NoLabel {
    fn label() -> &'static str {
        ""
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
        let encoded = base32_encode(&self.data, ALPHABET);
        format!("{}{}", T::label(), &encoded[SHORT_LENGTH..])
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

impl<T: Label> Debug for Oid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let encoded = base32_encode(&self.data, ALPHABET);
        write!(f, "{}{}_{}",
               T::label(),
               &encoded[..SHORT_LENGTH],
               &encoded[SHORT_LENGTH..],
        )
    }
}

pub trait Label {
    fn label() -> &'static str;

    #[cfg(all(feature = "rand", any(feature = "chrono", feature = "time")))]
    fn oid() -> Oid<Self> where Self: Sized {
        Oid::from_timestamp_with_rng(unix_epoch_ms(), &mut rand::thread_rng())
    }

    fn from_bytes(bytes: [u8; 16]) -> Oid<Self> where Self: Sized {
        Oid {
            data: bytes,
            marker: Default::default(),
        }
    }

    fn null() -> Oid<Self> where Self: Sized {
        Oid {
            data: [0; 16],
            marker: Default::default(),
        }
    }
}

macro_rules! label {
    ($name:ident, $label:literal) => {
        pub struct $name;
        impl Label for $name {
            fn label() -> &'static str {
                concat!($label, "_")
            }
        }
    };
}

impl<T: Label> Display for Oid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
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
}