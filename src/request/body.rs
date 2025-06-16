use thiserror::Error;

use crate::request::header::ContentType;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid bytes: {0:?}")]
    InvalidBytes(#[from] std::string::FromUtf8Error),

    #[error("Invalid JSON: {0:?}")]
    InvalidJson(#[from] serde_json::Error),
}

#[derive(Debug, Default)]
pub struct Body(BodyType);

#[derive(Debug, Clone)]
pub enum BodyType {
    TextPlain(String),
    ApplicationJson(serde_json::Value),
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::TextPlain(String::new())
    }
}

impl Body {
    pub fn parse(body_data: Vec<u8>, content_type: &ContentType) -> Result<Self, ParseError> {
        let body_str = String::from_utf8(body_data)?;

        let body = match content_type {
            ContentType::TextPlain => Self(BodyType::TextPlain(body_str)),
            ContentType::ApplicationJson => {
                Self(BodyType::ApplicationJson(serde_json::from_str(&body_str)?))
            }
        };

        Ok(body)
    }
}
