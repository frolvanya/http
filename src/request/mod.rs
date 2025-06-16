use std::collections::HashMap;

use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

mod body;
mod header;

const HTTP_PREFIX: &str = "HTTP";
const FRONT_SLASH_PREFIX: &str = "/";

macro_rules! parse_required_field {
    ($map:expr, $key:expr, $type:path) => {{
        let raw = $map.remove($key).ok_or_else(|| {
            RequestMessageError::MissingHeader(format!("Missing header: {}", $key))
        })?;
        <$type>::parse(&raw)?
    }};
}

#[derive(Error, Debug)]
pub enum RequestMessageError {
    #[error("Missing header: {0:?}")]
    MissingHeader(String),

    #[error("Failed to read from buffer: {0:?}")]
    ReadBufferError(#[from] std::io::Error),

    #[error("Failed to covert to utf8: {0:?}")]
    Utf8ConversionError(#[from] std::str::Utf8Error),

    #[error("Failed to parse request: {0:?}")]
    RequestParseError(#[from] ParseError),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse structure: {0:?}")]
    StructParseError(String),

    #[error("Unexpected entry: {0:?}")]
    UnexpectedEntry(String),
}

trait Parse: Sized {
    type Sanitized;

    fn sanitize(data: &str) -> Result<Self::Sanitized, ParseError>;
    fn parse(data: &str) -> Result<Self, ParseError>;
}

#[derive(Debug)]
pub struct RequestMessage {
    request_line: RequestLine,
    header: header::Header,
    body: body::Body,
}

impl RequestMessage {
    fn new(request_line: RequestLine, header: header::Header, body: body::Body) -> Self {
        Self {
            request_line,
            header,
            body,
        }
    }
}

#[derive(Debug)]
pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
}

impl Parse for RequestType {
    type Sanitized = RequestType;

    fn sanitize(data: &str) -> Result<Self::Sanitized, ParseError> {
        Ok(match data {
            "GET" => RequestType::Get,
            "POST" => RequestType::Post,
            "PUT" => RequestType::Put,
            "DELETE" => RequestType::Delete,
            unknown => {
                return Err(ParseError::UnexpectedEntry(format!(
                    "Found unexpected request type: {unknown}"
                )))
            }
        })
    }

    fn parse(data: &str) -> Result<Self, ParseError> {
        Self::sanitize(data)
    }
}

#[derive(Debug)]
pub struct RequestUri {
    inner: String,
}

impl Parse for RequestUri {
    type Sanitized = String;

    fn sanitize(data: &str) -> Result<Self::Sanitized, ParseError> {
        if data.is_empty() {
            return Err(ParseError::UnexpectedEntry(
                "Empty path was provided".to_owned(),
            ));
        }

        Ok(data.to_owned())
    }

    fn parse(data: &str) -> Result<RequestUri, ParseError> {
        Ok(Self {
            inner: Self::sanitize(data)?.to_owned(),
        })
    }
}

#[derive(Debug)]
pub enum HttpVersionEnum {
    V1_1,
}

#[derive(Debug)]
pub struct HttpVersion {
    inner: HttpVersionEnum,
}

impl Parse for HttpVersion {
    type Sanitized = String;

    fn sanitize(data: &str) -> Result<Self::Sanitized, ParseError> {
        let sanitized_data = data
            .trim()
            .strip_prefix(HTTP_PREFIX)
            .ok_or_else(|| {
                ParseError::UnexpectedEntry(format!(
                    "Failed to parse http prefix in HttpVersion: {data}"
                ))
            })?
            .strip_prefix(FRONT_SLASH_PREFIX)
            .ok_or_else(|| {
                ParseError::UnexpectedEntry(format!(
                    "Failed to strip front slash in http version: {data}"
                ))
            })?;

        Ok(sanitized_data.to_owned())
    }

    fn parse(data: &str) -> Result<Self, ParseError> {
        match Self::sanitize(data)?.as_str() {
            "1.1" => Ok(HttpVersion {
                inner: HttpVersionEnum::V1_1,
            }),
            err_version => Err(ParseError::StructParseError(format!(
                "HTTP version {err_version} is not supported"
            ))),
        }
    }
}

#[derive(Debug)]
pub struct RequestLine {
    pub request_type: RequestType,
    pub uri: RequestUri,
    pub version: HttpVersion,
}

impl Parse for RequestLine {
    type Sanitized = Vec<String>;

    fn sanitize(data: &str) -> Result<Self::Sanitized, ParseError> {
        // GET /index.html HTTP
        let sanitized_data = data
            .split_ascii_whitespace()
            .map(|component| component.to_owned())
            .collect::<Vec<_>>();

        if sanitized_data.len() != 3 {
            return Err(ParseError::UnexpectedEntry(format!(
                "Number of components in request line is not equal to 3: {data}"
            )));
        }

        Ok(sanitized_data)
    }

    fn parse(data: &str) -> Result<Self, ParseError> {
        let sanitized_data = Self::sanitize(data)?;

        Ok(Self {
            request_type: RequestType::parse(&sanitized_data[0])?,
            uri: RequestUri::parse(&sanitized_data[1])?,
            version: HttpVersion::parse(&sanitized_data[2])?,
        })
    }
}

#[tracing::instrument(name = "handle")]
pub async fn parse_request(stream: &mut TcpStream) -> Result<RequestMessage, RequestMessageError> {
    let mut reader = BufReader::new(stream);

    let mut raw_headers = HashMap::new();

    let mut request_line = None;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        if line == "\r\n" {
            break;
        }

        if let Ok(parsed_request_line) = RequestLine::parse(line.trim_end()) {
            if request_line.is_none() {
                request_line = Some(parsed_request_line);
            } else {
                return Err(
                    RequestMessageError::RequestParseError(
                        ParseError::UnexpectedEntry(
                            format!("Found multiple request lines\nNew: {parsed_request_line:?}\nCurrent: {request_line:?}")
                        )
                    )
                );
            }
        } else if let Some((key, value)) = line.trim_end().split_once(":") {
            raw_headers.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    let Some(request_line) = request_line else {
        return Err(RequestMessageError::RequestParseError(
            ParseError::UnexpectedEntry("No request line was found in request".to_owned()),
        ));
    };

    tracing::info!("Request Line: {request_line:?}");

    let header = header::Header::new(
        parse_required_field!(raw_headers, "Host", header::Host),
        parse_required_field!(raw_headers, "User-Agent", header::UserAgent),
        parse_required_field!(raw_headers, "Content-Type", header::ContentType),
        parse_required_field!(raw_headers, "Content-Length", header::ContentLength),
        raw_headers.into(),
    );

    tracing::info!("Header: {header:?}");

    let mut body = vec![
        0u8;
        usize::try_from(header.content_length()).map_err(|err| {
            RequestMessageError::RequestParseError(ParseError::UnexpectedEntry(format!(
                "Failed to parse content length as `usize`: {}, due to {err:?}",
                header.content_length()
            )))
        })?
    ];

    let body = if header.content_length() > 0 {
        reader.read_exact(&mut body).await?;
        body::Body::parse(&body, header.content_type())?
    } else {
        body::Body::default()
    };

    Ok(RequestMessage::new(request_line, header, body))
}

pub async fn handle(mut stream: TcpStream) -> Result<RequestMessage, RequestMessageError> {
    let request = parse_request(&mut stream).await?;

    // TODO: add an option to return html files
    let response_body = format!(
        "Hello, world!\nHere is your request printed:\n{}",
        request.body
    );

    // TODO: Make response builder
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
        response_body.len()
    );

    stream.write_all(response.as_bytes()).await?;
    stream.write_all(response_body.as_bytes()).await?;
    stream.flush().await?;

    Ok(request)
}
