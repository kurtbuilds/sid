use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub use label::Label;
pub use sid_encode::DecodeError;
use sid_encode::{base32_decode, base32_encode, SHORT_LENGTH};

mod label;
mod monotonic;
#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "sqlx")]
mod sqlx;

pub use monotonic::MonotonicGenerator;

#[cfg(target_arch = "wasm32")]
fn unix_epoch_millis() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(target_arch = "wasm32"))]
fn unix_epoch_millis() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[derive(Copy, Clone)]
pub struct NoLabel;

impl NoLabel {
    pub fn sid() -> Sid {
        Sid::<NoLabel>::new()
    }
}

pub struct Sid<T = NoLabel> {
    data: [u8; 16],
    marker: std::marker::PhantomData<T>,
}

impl<T: Label> std::hash::Hash for Sid<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

impl<T> PartialEq<Self> for Sid<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> PartialOrd<Self> for Sid<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for Sid<T> {}

impl<T> Ord for Sid<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl<T> Clone for Sid<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            marker: Default::default(),
        }
    }
}

impl<T> Copy for Sid<T> {}

pub fn sid<T: Label>() -> Sid<T> {
    Sid::<T>::new()
}

impl<T> Sid<T> {
    #[cfg(feature = "uuid")]
    pub fn uuid(&self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.data)
    }
}

impl<T: Label> Sid<T> {
    pub fn null() -> Self {
        Self {
            data: [0; 16],
            marker: Default::default(),
        }
    }

    pub fn new() -> Self {
        Self::from_timestamp_with_rng(unix_epoch_millis(), &mut rand::rng())
    }

    #[cfg(feature = "rand")]
    pub fn from_timestamp_with_rng<R>(timestamp: u64, rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        if (timestamp >> 48) != 0 {
            panic!("sid does not support timestamps after +10889-08-02T05:31:50.655Z");
        }
        let rand_high = rng.random::<u32>() as u64 & ((1 << 16) - 1);
        let high = timestamp << 16 | rand_high;
        let low = rng.random::<u64>();
        let high = high.to_be_bytes();
        let low = low.to_be_bytes();

        let mut data: [u8; 16] = [0; 16];
        data[..8].copy_from_slice(&high);
        data[8..].copy_from_slice(&low);

        Self {
            data,
            marker: Default::default(),
        }
    }

    /// Only the short suffix of the sid, with label, e.g. usr_t40x
    pub fn short(&self) -> String {
        let encoded = base32_encode(self.data);
        let label = T::label();
        let separator = if label.is_empty() { "" } else { "_" };
        format!("{}{}{}", label, separator, &encoded[SHORT_LENGTH + 1..])
    }

    pub fn is_null(&self) -> bool {
        self.data.iter().all(|&b| b == 0)
    }

    pub fn data(&self) -> &[u8; 16] {
        &self.data
    }

    pub fn timestamp(&self) -> u64 {
        u64::from_be_bytes(self.data[0..8].try_into().unwrap())
    }

    // small difference compared to ULID. Rather than erroring if we overflow the random buffer
    // we just increment the ms stamp.
    pub(crate) fn increment(&self) -> Self {
        let mut data = self.data;
        let mut i = 15;
        loop {
            let (value, overflow) = data[i].overflowing_add(1);
            data[i] = value;
            if !overflow {
                break;
            }
            if i == 0 {
                panic!("sid overflow");
            }
            i -= 1;
        }
        Self {
            data,
            marker: Default::default(),
        }
    }

    pub fn into_bytes(self) -> [u8; 16] {
        self.data
    }
}

#[cfg(feature = "uuid")]
impl<T> Into<uuid::Uuid> for Sid<T> {
    fn into(self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.data)
    }
}

#[cfg(feature = "uuid")]
impl<T> From<uuid::Uuid> for Sid<T> {
    fn from(value: uuid::Uuid) -> Self {
        let bytes = value.as_ref();
        let mut data: [u8; 16] = [0; 16];
        data.copy_from_slice(bytes);
        Self {
            data,
            marker: Default::default(),
        }
    }
}

impl<T> Sid<T> {
    pub fn unlabel(self) -> Sid<NoLabel> {
        Sid {
            data: self.data,
            marker: Default::default(),
        }
    }
}

impl Sid<NoLabel> {
    pub fn into_labeled<U>(self) -> Sid<U> {
        Sid {
            data: self.data,
            marker: Default::default(),
        }
    }
}

impl<T> From<[u8; 16]> for Sid<T> {
    fn from(data: [u8; 16]) -> Self {
        Self {
            data,
            marker: Default::default(),
        }
    }
}

impl<T> FromStr for Sid<T> {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = base32_decode(s)?;
        Ok(Sid::<T>::from(data))
    }
}

impl<T: Label> Debug for Sid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let encoded = base32_encode(self.data);
        let label = T::label();
        let sep = if label.is_empty() { "" } else { "_" };
        f.write_str(label)?;
        f.write_str(sep)?;
        f.write_str(&encoded)
    }
}

impl<T> Display for Sid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let encoded = base32_encode(self.data);
        f.write_str(&encoded)
    }
}

#[cfg(feature = "fake")]
impl<T: Label> fake::Dummy<fake::Faker> for Sid<T> {
    fn dummy(_: &fake::Faker) -> Self {
        Self::null()
    }

    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &fake::Faker, _: &mut R) -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! sid {
    ($value:ident) => {
        sid!(stringify!($value))
    };
    ($value:expr) => {{
        let value = $value;
        let count = value.matches('_').count();
        match count {
            1 => Sid::from_str(value).unwrap(),
            2 => {
                let value = value.splitn(2, '_').nth(1).unwrap();
                Sid::from_str(value).unwrap()
            }
            _ => panic!("sid must have 1 or 2 underscores."),
        }
    }};
}

#[cfg(test)]
mod tests {
    use label::Label;

    use super::*;

    label!(Team, "team");

    #[test]
    fn test_struct_sid_can_reference_itself() {
        struct Team {
            id: Sid<Self>,
        }

        impl Label for Team {
            fn label() -> &'static str {
                "tea_"
            }
        }

        let f = Team { id: sid() };
        let s = format!("{:?}", f.id);
        assert!(s.starts_with("tea_"));
    }

    #[test]
    fn it_works() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let sid = Sid::<Team>::from(bytes);
        println!("{}", sid.short());
        println!("{}", sid);
        assert_eq!(sid.to_string(), "01081g81860w40j2gb1g6g_w3rg");
        assert_eq!(sid.short(), "team_w3rg");
    }

    #[test]
    fn test_null() {
        let sid = Sid::<Team>::null();
        println!("{}", sid.short());
        println!("{}", sid);
        assert_eq!(sid.to_string(), "0000000000000000000000_0000");
        assert_eq!(sid.short(), "team_0000");
        let sid = Sid::<NoLabel>::null();
        assert_eq!(sid.to_string(), "0000000000000000000000_0000");
    }

    #[test]
    #[cfg(feature = "uuid")]
    fn test_uuid() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let sid = Sid::<Team>::from(bytes);
        let uuid: uuid::Uuid = sid.clone().into();
        assert_eq!(uuid.to_string(), "01020304-0506-0708-090a-0b0c0d0e0f10");
        let uuid2 = sid.uuid();
        assert_eq!(uuid, uuid2);
    }

    #[test]
    fn test_macro() {
        let sid: Sid<Team> = sid!("team_0000000000000000000000_0000");
        assert!(sid.is_null(), "{}", sid);
        let sid: Sid<Team> = sid!(team_0000000000000000000000_0000);
        assert!(sid.is_null(), "{}", sid);
        let sid: Sid<Team> = sid!(team_0da0fa0e02cssbhkanf04c_srb0);
        assert_eq!(sid.to_string(), "0da0fa0e02cssbhkanf04c_srb0");
    }

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<Sid<Team>>(), 16);
    }

    #[test]
    fn test_sort() {
        let ts = unix_epoch_millis();
        let ts2 = ts + 1;
        let ts3 = ts + 2;
        let rng = &mut rand::thread_rng();
        let sid1 = Sid::<NoLabel>::from_timestamp_with_rng(ts, rng);
        let sid2 = Sid::from_timestamp_with_rng(ts2, rng);
        let sid3 = Sid::from_timestamp_with_rng(ts3, rng);
        let mut sids = vec![sid3.clone(), sid1.clone(), sid2.clone()];
        sids.sort();
        assert_eq!(sids, vec![sid1, sid2, sid3]);
    }
}
