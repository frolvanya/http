use url::Url;

use std::collections::HashMap;

use crate::request::{Parse, ParseError};

#[derive(Debug)]
pub struct Header {
    host: Host,
    user_agent: UserAgent,
    content_type: ContentType,
    content_length: ContentLength,
    other_headers: OtherHeaders,
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

    pub fn content_type(&self) -> &ContentType {
        &self.content_type
    }

    pub fn content_length(&self) -> u64 {
        self.content_length.inner
    }
}

#[derive(Debug)]
pub struct Host {
    pub inner: String,
}

impl Parse for Host {
    type Sanitized = String;

    fn sanitize(data: &str) -> Result<Self::Sanitized, ParseError> {
        if let Err(err) = Url::parse(data) {
            return Err(ParseError::UnexpectedEntry(format!(
                "Invalid host provided: {data}, due to: {err:?}"
            )));
        }

        Ok(data.to_owned())
    }

    fn parse(data: &str) -> Result<Self, ParseError> {
        Ok(Self {
            inner: Self::sanitize(data)?.to_owned(),
        })
    }
}

#[derive(Debug)]
pub struct UserAgent {
    pub inner: String,
}

impl Parse for UserAgent {
    type Sanitized = String;

    fn sanitize(data: &str) -> Result<Self::Sanitized, ParseError> {
        if data.is_empty() {
            return Err(ParseError::UnexpectedEntry(
                "Invalid user agent was provided: {data}".to_owned(),
            ));
        }

        Ok(data.to_owned())
    }

    fn parse(data: &str) -> Result<Self, ParseError> {
        Ok(Self {
            inner: Self::sanitize(data)?.to_owned(),
        })
    }
}

#[derive(Debug)]
pub enum ContentType {
    TextType(Text),
    ApplicationType(Application),
}

#[derive(Debug)]
pub enum Text {
    Plain,
}

#[derive(Debug)]
pub enum Application {
    Json,
}

impl Parse for ContentType {
    type Sanitized = String;

    fn sanitize(data: &str) -> Result<Self::Sanitized, ParseError> {
        data.split(';')
            .next()
            .ok_or_else(|| {
                ParseError::UnexpectedEntry("Content-type is empty in data: {data}".to_owned())
            })
            .map(|data| data.to_owned())
    }

    fn parse(data: &str) -> Result<Self, ParseError> {
        match Self::sanitize(data)?.as_str() {
            "text/plain" => Ok(ContentType::TextType(Text::Plain)),
            "application/json" => Ok(ContentType::ApplicationType(Application::Json)),
            unknown => Err(ParseError::StructParseError(format!(
                "This application type is not supported: {unknown}"
            ))),
        }
    }
}

#[derive(Debug)]
pub struct ContentLength {
    inner: u64,
}

impl Parse for ContentLength {
    type Sanitized = u64;

    fn sanitize(data: &str) -> Result<Self::Sanitized, ParseError> {
        data.parse::<u64>().map_err(|err| {
            ParseError::UnexpectedEntry(format!(
                "Failed to parse {data} as Content Length, due to {err:?}"
            ))
        })
    }

    fn parse(data: &str) -> Result<Self, ParseError> {
        Ok(Self {
            inner: Self::sanitize(data)?,
        })
    }
}

#[derive(Debug)]
pub struct OtherHeaders {
    inner: HashMap<String, String>,
}

impl From<HashMap<String, String>> for OtherHeaders {
    fn from(value: HashMap<String, String>) -> Self {
        Self { inner: value }
    }
}
