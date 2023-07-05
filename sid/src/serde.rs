use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::fmt;
use serde::de::{self, Visitor};
use crate::{NoLabel, Sid};

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
                let sid: Self::Value = value.parse().map_err(E::custom)?;
                Ok(sid)
            }
        }

        deserializer.deserialize_str(SidVisitor).map(|sid| Sid::<T>::from(*sid.data()))
    }
}