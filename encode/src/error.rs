use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum DecodeError {
    InvalidCharacter(char),
    InvalidLength,
    NoSeparator,
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::InvalidCharacter(c) => write!(f, "Invalid character: {}", c),
            DecodeError::InvalidLength => write!(f, "Invalid length"),
            DecodeError::NoSeparator => write!(f, "No separator"),
        }
    }
}

impl std::error::Error for DecodeError {}
