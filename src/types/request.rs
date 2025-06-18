use super::{body, header, request_line};

#[derive(Debug)]
pub struct RequestMessage {
    pub request_line: request_line::RequestLine,
    pub header: header::Header,
    pub body: body::Body,
}

impl RequestMessage {
    pub const fn new(
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
