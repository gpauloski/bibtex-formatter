use derive_more::From;

use crate::token::{Token, TokenInfo};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    ContentParseError(String),
    EndOfTokenStream,
    MissingCiteKey(TokenInfo),
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
            Self::EndOfTokenStream => write!(fmt, "Unexpected end of token stream."),
            Self::UnexpectedToken(expected, found) => {
                write!(fmt, "Expected {:?} but found {:?}.", expected, found)
            }
            _ => write!(fmt, "{self:?}"),
        }
    }
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}
