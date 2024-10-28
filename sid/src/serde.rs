use crate::{NoLabel, Sid};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

impl<T> Serialize for Sid<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T> Deserialize<'de> for Sid<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SidVisitor;

        impl<'de> Visitor<'de> for SidVisitor {
            type Value = Sid<NoLabel>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sid (27 chars, crockford base32)")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let deserialize_uuid = cfg!(feature = "deserialize_uuid_strings");
                let sid: Self::Value = if deserialize_uuid && value.len() == 36 {
                    use std::str::FromStr;
                    let uuid = uuid::Uuid::from_str(value).map_err(E::custom)?;
                    Self::Value::from(uuid)
                } else {
                    value.parse().map_err(E::custom)?
                };
                Ok(sid)
            }
        }
        let value = deserializer.deserialize_str(SidVisitor)?;
        let value = value.into_bytes();
        Ok(Sid::<T>::from(value))
    }
}
