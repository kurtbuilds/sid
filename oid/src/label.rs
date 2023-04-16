use crate::{NoLabel, Oid};

impl Label for NoLabel {
    fn label() -> &'static str {
        ""
    }
}

pub trait Label {
    fn label() -> &'static str;

    #[cfg(all(feature = "rand", any(feature = "chrono", feature = "time")))]
    fn oid() -> Oid<Self> where Self: Sized {
        Oid::from_timestamp_with_rng(crate::unix_epoch_ms(), &mut rand::thread_rng())
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

#[macro_export]
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