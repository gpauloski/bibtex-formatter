use derive_more::From;

use crate::token::{Position, Token, TokenInfo};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    EndOfTokenStream(Position),
    InternalAssertion(String),
    MissingCiteKey(TokenInfo),
    MissingContent(TokenInfo),
    MissingEntryType(TokenInfo),
    MissingTagName(TokenInfo),
    UnexpectedToken(Token, TokenInfo),

    #[from]
    Custom(String),

    #[from]
    Io(std::io::Error),
}

impl Error {
    pub fn custom(val: impl std::fmt::Display) -> Self {
        Self::Custom(val.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        match self {
            Self::EndOfTokenStream(position) => {
                write!(fmt, "Unexpected end of token stream at {}", position)
            }
            Self::InternalAssertion(message) => {
                write!(fmt, "Internal assertion error: {}", message)
            }
            Self::MissingCiteKey(info) => write!(
                fmt,
                "Expected cite key at {}; found `{}`",
                info.position, info.value
            ),
            Self::MissingContent(info) => write!(
                fmt,
                "Expected tag content at {}; found `{}`",
                info.position, info.value,
            ),
            Self::MissingEntryType(info) => write!(
                fmt,
                "Expected entry type at {}; found `{}`",
                info.position, info.value,
            ),
            Self::MissingTagName(info) => write!(
                fmt,
                "Expected tag name at {}; found `{}`",
                info.position, info.value,
            ),
            Self::UnexpectedToken(expected, found) => write!(
                fmt,
                "Expected `{}` at {}; found `{}`",
                expected, found.position, found.value,
            ),
            _ => write!(fmt, "{self:?}"),
        }
    }
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}
