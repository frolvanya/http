use super::{request_line, status};

#[derive(Debug)]
pub struct ResponseLine {
    pub http_version: request_line::HttpVersion,
    pub status: status::Status,
}

impl ResponseLine {
    pub fn new(http_version: request_line::HttpVersion, status: status::Status) -> Self {
        Self {
            http_version,
            status,
        }
    }
}

impl std::fmt::Display for ResponseLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.http_version,
            self.status.status_code(),
            self.status,
        )
    }
}
