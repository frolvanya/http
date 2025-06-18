use std::collections::HashMap;

use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

mod body;
mod header;
mod request_line;

#[derive(Error, Debug)]
pub enum RequestMessageError {
    #[error("Request line not found in request")]
    RequestLineNotFound,

    #[error("Multiple request lines found: {0}")]
    MultipleRequestLines(String),

    #[error("Failed to read from buffer: {0:?}")]
    ReadBufferError(#[from] std::io::Error),

    #[error("Failed to covert to utf8: {0:?}")]
    Utf8ConversionError(#[from] std::str::Utf8Error),

    #[error("Request line parse error: {0:?}")]
    RequestLineParseError(#[from] request_line::ParseError),

    #[error("Header parse error: {0:?}")]
    HeaderParseError(#[from] header::ParseError),

    #[error("Body parse error: {0:?}")]
    BodyParseError(#[from] body::ParseError),
}

#[derive(Debug)]
pub struct RequestMessage {
    request_line: request_line::RequestLine,
    header: header::Header,
    body: body::Body,
}

impl RequestMessage {
    fn new(
        request_line: request_line::RequestLine,
        header: header::Header,
        body: body::Body,
    ) -> Self {
        Self {
            request_line,
            header,
            body,
        }
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

        line = line.trim().to_ascii_lowercase();

        if line.is_empty() {
            break;
        }

        if let Ok(parsed_request_line) = line.parse() {
            if request_line.is_none() {
                request_line = Some(parsed_request_line);
            } else {
                return Err(
                    RequestMessageError::MultipleRequestLines(
                        format!("Found multiple request lines\nNew: {parsed_request_line:?}\nCurrent: {request_line:?}")
                    )
                );
            }
        } else if let Some((key, value)) = line.split_once(":") {
            raw_headers.insert(key.trim_end().to_string(), value.trim_start().to_string());
        }
    }

    let Some(request_line) = request_line else {
        return Err(RequestMessageError::RequestLineNotFound);
    };

    let header = header::Header::try_from(&mut raw_headers)?;

    let content_length = header.content_length.try_into()?;

    let body = if content_length > 0 {
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body).await?;
        body::Body::parse(body, &header.content_type)?
    } else {
        body::Body::default()
    };

    Ok(RequestMessage::new(request_line, header, body))
}

pub async fn handle(mut stream: TcpStream) -> Result<RequestMessage, RequestMessageError> {
    let request_message = parse_request(&mut stream).await?;

    // TODO: add an option to return html files
    let response_body = format!(
        "Hello, world!\nHere is your request printed:\n{:?}",
        request_message.body
    );

    // TODO: Make response builder
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
        response_body.len()
    );

    stream.write_all(response.as_bytes()).await?;
    stream.write_all(response_body.as_bytes()).await?;
    stream.flush().await?;

    Ok(request_message)
}
