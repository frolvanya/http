use super::{body, header, response_line};

#[derive(Debug)]
pub struct ResponseMessage {
    pub response_line: response_line::ResponseLine,
    pub header: header::Header,
    pub body: body::Body,
}

impl ResponseMessage {
    pub fn new(
        response_line: response_line::ResponseLine,
        header: header::Header,
        body: body::Body,
    ) -> Self {
        Self {
            response_line,
            header,
            body,
        }
    }
}

impl std::fmt::Display for ResponseMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\r\n{}\r\n\r\n{}",
            self.response_line, self.header, self.body,
        )
    }
}
