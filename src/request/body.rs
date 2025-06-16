use super::{
    header::{Application, ContentType, Text},
    ParseError,
};

#[derive(Debug, Default, Clone)]
pub struct Body {
    pub inner: Vec<u8>,
}

impl From<&Vec<u8>> for Body {
    fn from(value: &Vec<u8>) -> Self {
        Self {
            inner: value.clone(),
        }
    }
}

impl From<&str> for Body {
    fn from(value: &str) -> Self {
        Self {
            inner: value.as_bytes().to_vec().clone(),
        }
    }
}

impl std::fmt::Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            std::str::from_utf8(&self.inner).expect("Vec of valid bytes")
        )
    }
}

impl Body {
    pub fn parse(body_data: &Vec<u8>, content_type: &ContentType) -> Result<Self, ParseError> {
        match content_type {
            ContentType::TextType(Text::Plain) => Ok(Self::from(body_data)),
            // TODO: parse using serde or smth
            ContentType::ApplicationType(Application::Json) => Ok(Self::from(body_data)),
        }
    }
}
