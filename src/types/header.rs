use std::{collections::HashMap, str::FromStr};

use thiserror::Error;
use url::Url;

const HOST_HEADER_NAME: &str = "host";
const CONTENT_TYPE_HEADER_NAME: &str = "content-type";
const CONTENT_LENGTH_HEADER_NAME: &str = "content-length";

macro_rules! parse_required_field {
    ($map:expr, $key:expr, $type:path) => {{
        let raw = $map
            .remove($key)
            .ok_or_else(|| ParseError::MissingHeader(format!("Missing header: {}", $key)))?;
        raw.parse()?
    }};
}

macro_rules! parse_optional_field {
    ($map:expr, $key:expr, $type:path, $default:expr) => {{
        $map.remove($key)
            .map(|v| v.parse())
            .unwrap_or(Ok($default))?
    }};
}

fn capitalize(word: &str) -> String {
    word.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("-")
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
    pub content_type: ContentType,
    pub content_length: ContentLength,
    pub other_headers: OtherHeaders,
}

impl Header {
    pub fn new(
        host: Host,
        content_type: ContentType,
        content_length: ContentLength,
        other_headers: OtherHeaders,
    ) -> Self {
        Self {
            host,
            content_type,
            content_length,
            other_headers,
        }
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\r\n{}\r\n{}\r\n{}",
            self.host, self.content_type, self.content_length, self.other_headers
        )
    }
}

impl TryFrom<&mut HashMap<String, String>> for Header {
    type Error = ParseError;

    fn try_from(value: &mut HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(Self {
            host: parse_required_field!(value, HOST_HEADER_NAME, Host),
            content_type: parse_optional_field!(
                value,
                CONTENT_TYPE_HEADER_NAME,
                ContentType,
                ContentType::TextPlain
            ),
            content_length: parse_optional_field!(
                value,
                CONTENT_LENGTH_HEADER_NAME,
                ContentLength,
                ContentLength::default()
            ),
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

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", capitalize(HOST_HEADER_NAME), self.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContentType {
    TextPlain,
    TextHtml,
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

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_type_str = match self {
            ContentType::TextPlain => "text/plain",
            ContentType::TextHtml => "text/html",
            ContentType::ApplicationJson => "application/json",
        };

        write!(
            f,
            "{}: {}",
            capitalize(CONTENT_TYPE_HEADER_NAME),
            content_type_str
        )
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ContentLength(u64);

impl ContentLength {
    pub fn new(content_length: u64) -> Self {
        Self(content_length)
    }
}

impl FromStr for ContentLength {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl std::fmt::Display for ContentLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", capitalize(CONTENT_LENGTH_HEADER_NAME), self.0)
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

impl std::fmt::Display for OtherHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(key, value)| format!("{}: {}", capitalize(key), value))
                .collect::<Vec<_>>()
                .join("\r\n")
        )
    }
}
