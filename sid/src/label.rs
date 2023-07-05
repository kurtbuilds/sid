use crate::{NoLabel};

impl Label for NoLabel {
    fn label() -> &'static str {
        ""
    }
}

pub trait Label {
    fn label() -> &'static str;
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