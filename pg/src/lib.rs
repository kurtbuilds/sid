use std::fmt::{Debug, Display};
use std::num::NonZeroU64;
use pgx::prelude::*;
use pgx::StringInfo;
use sid_encode::{base32_encode, base32_decode, SHORT_LENGTH};

pgx::pg_module_magic!();

const MAX_LABEL_LENGTH: usize = 8;

// A tiny string type, bounded size and implements Copy
#[derive(Copy, Clone, PartialEq)]
struct Label(NonZeroU64);

impl Label {
    // pub fn as_str(&self) -> &str {
    //     let bytes = &self.0.get().
    //     let &len = bytes.iter().find(|&&c| c == 0).unwrap_or(&(MAX_LABEL_LENGTH as u8));
    //     unsafe { std::str::from_utf8_unchecked(&bytes[..len as usize]) }
    // }

    pub fn new(label: &str) -> Option<Self> {
        if label.is_empty() {
            return None;
        }
        let len = label.len().min(MAX_LABEL_LENGTH);
        let bytes = label.as_bytes();
        let mut buf = [0u8; MAX_LABEL_LENGTH];
        buf[..len].copy_from_slice(&bytes[..len]);
        let data = u64::from_le_bytes(buf);
        Some(Label(NonZeroU64::new(data).unwrap()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        let len = bytes.len().min(MAX_LABEL_LENGTH);
        let mut buf = [0u8; MAX_LABEL_LENGTH];
        buf[..len].copy_from_slice(&bytes[..len]);
        let data = u64::from_le_bytes(buf);
        Some(Label(NonZeroU64::new(data)?))
    }
}

impl Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.get().to_le_bytes();
        write!(f, "Label({:?})", bytes)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.get().to_le_bytes();
        let &len = bytes.iter().find(|&&c| c == 0).unwrap_or(&(MAX_LABEL_LENGTH as u8));
        let s = unsafe { std::str::from_utf8_unchecked(&bytes[..len as usize]) };
        write!(f, "{}", s)
    }
}

// impl FromStr for Label {
//     type Err = &'static str;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         Label::new(s)
//         if s.len() > MAX_LABEL_LENGTH {
//             Err("Label too long")
//         } else {
//         }
//     }
// }

// impl TryFrom<&[u8; 8]> for Label {
//     type Error = Utf8Error;
//
//     fn try_from(value: &[u8; 8]) -> Result<Self, Self::Error> {
//         let s = std::str::from_utf8(value)?;
//         Ok(Label::new(s))
//     }
// }

#[derive(Copy, Clone, PostgresType)]
#[inoutfuncs]
#[derive(Debug)]
pub struct Sid {
    data: [u8; 16],
    label: Option<Label>,
}

impl Sid {
    pub fn new(data: [u8; 16], label: &str) -> Self {
        Self { data, label: Label::new(label) }
    }
}

impl InOutFuncs for Sid {
    fn input(input: &core::ffi::CStr) -> Sid {
        let input = input.to_str().unwrap();
        let (label, data) = match input.matches('_').count() {
            1 => {
                let data = base32_decode(input).unwrap();
                (None, data)
            }
            2 => {
                let mut value = input.splitn(2, '_');
                let label = value.next().unwrap().to_string();
                let data = base32_decode(value.next().unwrap()).unwrap();
                let label = Label::new(&label);
                (label, data)
            }
            _ => panic!("Invalid input"),
        };
        Sid { label, data }
    }

    // Output ourselves as text into the provided `StringInfo` buffer
    fn output(&self, buffer: &mut StringInfo) {
        if let Some(label) = &self.label {
            let bytes = label.0.get().to_le_bytes();
            for c in bytes {
                if c == 0 {
                    break;
                }
                buffer.push(c as char);
            }
            buffer.push_str("_");
        }
        let encoded = base32_encode(&self.data);
        buffer.push_str(&encoded[..SHORT_LENGTH]);
        buffer.push_str("_");
        buffer.push_str(&encoded[SHORT_LENGTH..]);
    }
}

impl serde::Serialize for Sid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        use serde::ser::SerializeTupleStruct;
        let first = u64::from_le_bytes(self.data[..8].try_into().unwrap());
        let second = u64::from_le_bytes(self.data[8..16].try_into().unwrap());

        let third: u64 = self.label.map(|l| l.0.get())
            .unwrap_or_default();

        let mut tup = serializer.serialize_tuple_struct("Oid", 3)?;
        tup.serialize_field(&first)?;
        tup.serialize_field(&second)?;
        tup.serialize_field(&third)?;
        tup.end()
    }
}

impl<'de> serde::Deserialize<'de> for Sid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use serde::de::SeqAccess;

        struct TupVisitor;

        impl<'de> Visitor<'de> for TupVisitor {
            type Value = Sid;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an array with 3 values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
            {
                let first: u64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let second: u64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let third: u64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let mut data: [u8; 16] = [0; 16];
                data[..8].copy_from_slice(&first.to_le_bytes());
                data[8..].copy_from_slice(&second.to_le_bytes());

                let label = if third == 0 {
                    None
                } else {
                    Some(Label::from_bytes(&third.to_le_bytes())
                        .ok_or_else(|| de::Error::custom("Could not decode label to string."))?)
                };
                Ok(Sid { data, label })
            }
        }

        deserializer.deserialize_tuple_struct("label", 3, TupVisitor)
    }
}

#[pg_extern]
fn sid_null(label: &str) -> Sid {
    Sid::new([0; 16], label)
}

#[pg_extern]
fn sid_new(label: &str) -> Sid {
    use std::time::SystemTime;
    use rand::Rng;

    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
    let rng = &mut ::rand::thread_rng();
    let high = timestamp << 16 | u64::from(rng.gen::<u16>());
    let low = rng.gen::<u64>();
    let high = high.to_le_bytes();
    let low = low.to_le_bytes();

    let mut data: [u8; 16] = [0; 16];
    data[..8].copy_from_slice(&high);
    data[8..].copy_from_slice(&low);

    Sid::new(data, label)
}

#[pg_extern]
fn sid_from_uuid(uuid: pgx::Uuid, label: &str) -> Sid {
    let &data = uuid.as_bytes();
    Sid::new(data, label)
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgx::prelude::*;
    use super::*;

    #[pg_test]
    fn test_sid_null() {
        let sid = sid_null("test");
        assert!(sid.data.iter().all(|&x| x == 0));
        assert_eq!(sid.label, Label::new("test"));
    }
}

/// This module is required by `cargo pgx test` invocations. 
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}

#[cfg(test)]
mod rust_tests {
    use super::*;

    #[test]
    fn test_zst() {
        assert_eq!(std::mem::size_of::<Label>(), MAX_LABEL_LENGTH);
        assert_eq!(std::mem::size_of::<Option<Label>>(), MAX_LABEL_LENGTH);
        assert_eq!(std::mem::size_of::<Sid>(), MAX_LABEL_LENGTH + 16);
    }

    #[test]
    fn test_serialize() {
        let sid = Sid { data: [0; 16], label: Label::new("team") };
        let serialized = serde_json::to_string(&sid).unwrap();
        println!("{}", serialized);
        assert_eq!(serialized, "[0,0,1835099508]");
    }

    #[test]
    fn test_label() {
        let label = Label::new("test").unwrap();
        let data = label.0.get().to_le_bytes();
        assert_eq!(data[0], b't');
        assert_eq!(data[1], b'e');
        assert_eq!(data[2], b's');
        assert_eq!(data[3], b't');
        assert_eq!(data[4], b'\0');
    }
}