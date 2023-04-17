use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub use label::Label;
use sid_encode::{base32_encode, base32_decode, SHORT_LENGTH};
pub use sid_encode::DecodeError;

mod label;


fn unix_epoch_ms() -> u64 {
    use std::time::SystemTime;
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64
}

pub struct NoLabel;

#[derive(PartialOrd, PartialEq, Eq, Ord, Hash, Copy)]
pub struct Sid<T = NoLabel> {
    data: [u8; 16],
    marker: std::marker::PhantomData<T>,
}

impl<T> Clone for Sid<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            marker: Default::default(),
        }
    }
}

pub fn sid<T: Label>() -> Sid<T> {
    T::sid()
}

impl<T: Label> Sid<T> {
    #[cfg(feature = "rand")]
    pub fn from_timestamp_with_rng<R>(timestamp: u64, rng: &mut R) -> Self
        where
            R: rand::Rng,
    {
        if (timestamp & 0xFFFF_0000_0000_0000) != 0 {
            panic!("sid does not support timestamps after +10889-08-02T05:31:50.655Z");
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
impl<T> Into<uuid::Uuid> for Sid<T> {
    fn into(self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.data)
    }
}

#[cfg(feature = "uuid")]
impl<T: Label> From<uuid::Uuid> for Sid<T> {
    fn from(value: uuid::Uuid) -> Self {
        let bytes = value.as_ref();
        let mut data: [u8; 16] = [0; 16];
        data.copy_from_slice(bytes);
        T::from_bytes(data)
    }
}

impl<T: Label> FromStr for Sid<T> {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = base32_decode(s)?;
        Ok(T::from_bytes(data))
    }
}

impl<T: Label> Debug for Sid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let encoded = base32_encode(&self.data);
        write!(f, "{}_{}",
               &encoded[..SHORT_LENGTH],
               &encoded[SHORT_LENGTH..],
        )
    }
}

impl<T: Label> Display for Sid<T> {
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
            },
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
        assert!(f.id.to_string().starts_with("tea_"));
    }

    #[test]
    fn it_works() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let sid = Team::from_bytes(bytes);
        println!("{}", sid.short());
        println!("{}", sid);
        assert_eq!(sid.to_string(), "team_0da0fa0e02cssbhkanf04c_srb0");
        assert_eq!(sid.short(), "team_srb0");
    }

    #[test]
    fn test_null() {
        let sid = Team::null();
        println!("{}", sid.short());
        println!("{}", sid);
        assert_eq!(sid.to_string(), "team_0000000000000000000000_0000");
        assert_eq!(sid.short(), "team_0000");
        let sid = NoLabel::null();
        assert_eq!(sid.to_string(), "0000000000000000000000_0000");
    }

    #[test]
    #[cfg(feature = "uuid")]
    fn test_uuid() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let sid = Team::from_bytes(bytes);
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
        assert_eq!(sid.to_string(), "team_0da0fa0e02cssbhkanf04c_srb0");
    }

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<Sid<Team>>(), 16);
    }
}
