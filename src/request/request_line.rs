use std::str::FromStr;

use thiserror::Error;

const HTTP_PREFIX: &str = "HTTP";
const FRONT_SLASH_PREFIX: &str = "/";

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected entry in request line: {0}")]
    InvalidRequestLineLength(usize),

    #[error("Invalid request type provided: {0}")]
    InvalidRequestType(String),

    #[error("Empty path provided")]
    EmtpyPath,

    #[error("Invalid HTTP Version: {0}")]
    InvalidHttpVersion(String),
}

#[derive(Debug)]
pub struct RequestLine {
    request_type: RequestType,
    uri: Path,
    version: HttpVersion,
}

impl FromStr for RequestLine {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitized_s = s
            .split_ascii_whitespace()
            .map(|component| component.to_owned())
            .collect::<Vec<_>>();

        if sanitized_s.len() != 3 {
            return Err(Self::Err::InvalidRequestLineLength(sanitized_s.len()));
        }

        Ok(Self {
            request_type: sanitized_s[0].parse()?,
            uri: sanitized_s[1].parse()?,
            version: sanitized_s[2].parse()?,
        })
    }
}

#[derive(Debug)]
pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
}

impl FromStr for RequestType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "GET" => Self::Get,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            unknown => {
                return Err(Self::Err::InvalidRequestType(unknown.to_owned()));
            }
        })
    }
}

#[derive(Debug)]
pub struct Path(String);

impl FromStr for Path {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Self::Err::EmtpyPath);
        }

        Ok(Self(s.to_owned()))
    }
}

#[derive(Debug)]
pub enum HttpVersionEnum {
    V1_1,
}

#[derive(Debug)]
pub struct HttpVersion(HttpVersionEnum);

impl FromStr for HttpVersion {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitized_s = s
            .trim()
            .strip_prefix(HTTP_PREFIX)
            .ok_or_else(|| {
                Self::Err::InvalidHttpVersion(format!("Failed to parse http prefix: {s}"))
            })?
            .strip_prefix(FRONT_SLASH_PREFIX)
            .ok_or_else(|| {
                Self::Err::InvalidHttpVersion(format!("Failed to strip front slash: {s}"))
            })?;

        match sanitized_s {
            "1.1" => Ok(HttpVersion(HttpVersionEnum::V1_1)),
            unknown => Err(Self::Err::InvalidHttpVersion(format!(
                "HTTP version {unknown} is not supported"
            ))),
        }
    }
}
