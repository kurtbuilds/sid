use crate::{NoLabel, Sid};

impl Label for NoLabel {
    fn label() -> &'static str {
        ""
    }
}

pub trait Label {
    fn label() -> &'static str;

    #[cfg(feature = "rand")]
    fn sid() -> Sid<Self> where Self: Sized {
        Sid::from_timestamp_with_rng(crate::unix_epoch_ms(), &mut rand::thread_rng())
    }

    fn from_bytes(bytes: [u8; 16]) -> Sid<Self> where Self: Sized {
        Sid {
            data: bytes,
            marker: Default::default(),
        }
    }

    fn null() -> Sid<Self> where Self: Sized {
        Sid {
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
                concat!($label)
            }
        }
    };
}