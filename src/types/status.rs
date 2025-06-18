use std::str::FromStr;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unknown Status Code: {0}")]
    UnknownStatusCode(String),
}

// TODO: Maybe it's better to make it a wrapper around enum
#[derive(PartialEq, Eq, Debug)]
pub struct Status(u64);

impl Status {
    pub const OK_STATUS_NAME: &str = "OK";
    pub const NOT_FOUND_STATUS_NAME: &str = "Not found";
    pub const INTERNAL_SERVER_ERROR_STATUS_NAME: &str = "Internal Server Error";

    pub const OK: Self = Status(200);
    pub const NOT_FOUND: Self = Status(404);
    pub const INTERNAL_SERVER_ERROR: Self = Status(500);

    pub fn new(status_code: u64) -> Self {
        Self(status_code)
    }

    pub fn status_code(&self) -> u64 {
        self.0
    }
}

impl FromStr for Status {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::OK_STATUS_NAME => Ok(Self::OK),
            Self::NOT_FOUND_STATUS_NAME => Ok(Self::NOT_FOUND),
            Self::INTERNAL_SERVER_ERROR_STATUS_NAME => Ok(Self::INTERNAL_SERVER_ERROR),
            unknown => Err(ParseError::UnknownStatusCode(unknown.to_owned())),
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::OK => Self::OK_STATUS_NAME,
                Self::NOT_FOUND => Self::NOT_FOUND_STATUS_NAME,
                Self::INTERNAL_SERVER_ERROR => Self::INTERNAL_SERVER_ERROR_STATUS_NAME,
                _ => "<status code is unknown>",
            }
        )
    }
}
