use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::str::{FromStr, Utf8Error};
use pgx::prelude::*;
use pgx::StringInfo;
use sid_encode::{base32_encode, base32_decode, SHORT_LENGTH};

pgx::pg_module_magic!();

const MAX_LABEL_LENGTH: usize = 8;

// A tiny string type, bounded size and implements Copy
#[derive(Copy, Clone)]
struct Label([u8; MAX_LABEL_LENGTH]);

impl Label {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        let len = self.0.iter().find(|&&c| c == 0).unwrap_or(&(MAX_LABEL_LENGTH as u8));
        unsafe { std::str::from_utf8_unchecked(&self.0[..*len as usize]) }
    }

    pub fn new(label: &str) -> Self {
        let mut data = [0; MAX_LABEL_LENGTH];
        data[..label.len()].copy_from_slice(label.as_bytes());
        Label(data)
    }
}

impl Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Label({})", self.as_str())
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl FromStr for Label {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > MAX_LABEL_LENGTH {
            Err("Label too long")
        } else {
            Ok(Label::new(s))
        }
    }
}

impl TryFrom<&[u8; 8]> for Label {
    type Error = Utf8Error;

    fn try_from(value: &[u8; 8]) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(value)?;
        Ok(Label::new(s))
    }
}

#[derive(Copy, Clone, PostgresType)]
#[inoutfuncs]
#[derive(Debug)]
pub struct Sid {
    data: [u8; 16],
    label: Option<Label>,
}

impl Sid {
    pub fn null() -> Self {
        Self {
            data: [0; 16],
            label: None,
        }
    }

    pub fn null_user() -> Self {
        Self {
            data: [0; 16],
            label: Some(Label::new("user")),
        }
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
                (Some(label), data)
            }
            _ => panic!("Invalid input"),
        };
        Sid { label, data }
    }

    // Output ourselves as text into the provided `StringInfo` buffer
    fn output(&self, buffer: &mut StringInfo) {
        if let Some(label) = &self.label {
            for c in label.0 {
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
        let third = self.label.map(|l| u64::from_le_bytes(l.as_bytes().try_into().unwrap())).unwrap_or_default();

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
                    Some(Label::try_from(&third.to_le_bytes())
                        .map_err(|_| de::Error::custom("Could not decode label to string."))?)
                };
                Ok(Sid { data, label })
            }
        }

        deserializer.deserialize_tuple_struct("label", 3, TupVisitor)
    }
}

#[pg_extern]
fn null_sid() -> Sid {
    Sid::null()
}

#[pg_extern]
fn null_user_sid() -> Sid {
    Sid::null_user()
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgx::prelude::*;

    #[pg_test]
    fn test_hello_oid_pg() {
        assert_eq!("Hello, oid_pg", crate::hello_oid_pg());
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
        assert_eq!(std::mem::size_of::<Oid>(), MAX_LABEL_LENGTH + 16);
        assert_eq!(std::mem::size_of::<Option<Oid>>(), MAX_LABEL_LENGTH + 16);
    }

    #[test]
    fn test_serialize() {
        let oid = Sid { data: [0; 16], label: Some(Label::new("team")) };
        let serialized = serde_json::to_string(&oid).unwrap();
        println!("{}", serialized);
        assert_eq!(serialized, "[0,0,1835099508]");
    }
}