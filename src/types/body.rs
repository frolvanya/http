use thiserror::Error;

use super::header;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid bytes: {0:?}")]
    InvalidBytes(#[from] std::string::FromUtf8Error),

    #[error("Invalid JSON: {0:?}")]
    InvalidJson(#[from] serde_json::Error),
}

#[derive(Debug, Default)]
pub struct Body(BodyType);

impl Body {
    pub const fn new(body_type: BodyType) -> Self {
        Self(body_type)
    }

    pub const fn get_type(&self) -> &BodyType {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum BodyType {
    TextPlain(String),
    TextHtml(String),
    ApplicationJson(serde_json::Value),
}

impl std::fmt::Display for BodyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::TextPlain(text) => text.clone(),
                Self::TextHtml(html) => html.clone(),
                Self::ApplicationJson(json) => json.to_string(),
            }
        )
    }
}

impl Default for BodyType {
    fn default() -> Self {
        Self::TextPlain(String::new())
    }
}

impl Body {
    pub fn parse(
        body_data: Vec<u8>,
        content_type: &header::ContentType,
    ) -> Result<Self, ParseError> {
        let body_str = String::from_utf8(body_data)?;

        let body = match content_type {
            header::ContentType::TextPlain => Self(BodyType::TextPlain(body_str)),
            header::ContentType::TextHtml => Self(BodyType::TextHtml(body_str)),
            header::ContentType::ApplicationJson => {
                Self(BodyType::ApplicationJson(serde_json::from_str(&body_str)?))
            }
        };

        Ok(body)
    }
}

impl std::fmt::Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
