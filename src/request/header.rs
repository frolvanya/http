use thiserror::Error;
use url::Url;

use std::{collections::HashMap, str::FromStr};

macro_rules! parse_required_field {
    ($map:expr, $key:expr, $type:path) => {{
        let raw = $map
            .remove($key)
            .ok_or_else(|| ParseError::MissingHeader(format!("Missing header: {}", $key)))?;
        raw.parse()?
    }};
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Missing header: {0}")]
    MissingHeader(String),

    #[error("Invalid url provided: {0:?}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Failed to parse as number: {0:?}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Failed to parse as number: {0:?}")]
    TryFromIntError(#[from] std::num::TryFromIntError),

    #[error("Unsupported ContentType: {0}")]
    UnsupportedContentType(String),
}

#[derive(Debug)]
pub struct Header {
    pub host: Host,
    pub user_agent: UserAgent,
    pub content_type: ContentType,
    pub content_length: ContentLength,
    pub other_headers: OtherHeaders,
}

impl Header {
    pub fn new(
        host: Host,
        user_agent: UserAgent,
        content_type: ContentType,
        content_length: ContentLength,
        other_headers: OtherHeaders,
    ) -> Self {
        Self {
            host,
            user_agent,
            content_type,
            content_length,
            other_headers,
        }
    }
}

impl TryFrom<&mut HashMap<String, String>> for Header {
    type Error = ParseError;

    fn try_from(value: &mut HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(Self {
            host: parse_required_field!(value, "Host", Host),
            user_agent: parse_required_field!(value, "User-Agent", UserAgent),
            content_type: parse_required_field!(value, "Content-Type", ContentType),
            content_length: parse_required_field!(value, "Content-Length", ContentLength),
            other_headers: value.clone().into(),
        })
    }
}

#[derive(Debug)]
pub struct Host(String);

impl FromStr for Host {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Url::parse(s)?;

        Ok(Self(s.to_owned()))
    }
}

#[derive(Debug)]
pub struct UserAgent(String);

impl FromStr for UserAgent {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Self::Err::MissingHeader("User-Agent".to_owned()));
        }

        Ok(Self(s.to_owned()))
    }
}

#[derive(Debug)]
pub enum ContentType {
    TextPlain,
    ApplicationJson,
}

impl FromStr for ContentType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitized_s = s.split_once(';').map(|(before, _)| before).unwrap_or(s);

        match sanitized_s {
            "text/plain" => Ok(ContentType::TextPlain),
            "application/json" => Ok(ContentType::ApplicationJson),
            unknown => Err(Self::Err::UnsupportedContentType(format!(
                "Unsupported content type: {unknown}"
            ))),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ContentLength(u64);

impl FromStr for ContentLength {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl TryInto<usize> for ContentLength {
    type Error = ParseError;
    fn try_into(self) -> Result<usize, Self::Error> {
        self.0.try_into().map_err(ParseError::TryFromIntError)
    }
}

#[derive(Debug)]
pub struct OtherHeaders(HashMap<String, String>);

impl From<HashMap<String, String>> for OtherHeaders {
    fn from(value: HashMap<String, String>) -> Self {
        Self(value)
    }
}
