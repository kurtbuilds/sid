use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct DecodeError(pub(crate) String);

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DecodeError: {}", self.0)
    }
}

impl std::error::Error for DecodeError {}
