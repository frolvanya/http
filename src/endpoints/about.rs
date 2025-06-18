use crate::types::{
    body::Body,
    header::{ContentLength, ContentType},
    request::RequestMessage,
    request_line::{HttpVersion, HttpVersionEnum, RequestType},
    response::ResponseMessage,
    response_line::ResponseLine,
    status::Status,
};

pub async fn handle(mut request_message: RequestMessage) -> ResponseMessage {
    if request_message.request_line.request_type != RequestType::Get {
        let reponse_line =
            ResponseLine::new(HttpVersion::new(HttpVersionEnum::V1_1), Status::NOT_FOUND);

        return ResponseMessage::new(reponse_line, request_message.header, Body::default());
    }

    let Ok(contents) = tokio::fs::read_to_string("pages/about.html").await else {
        let reponse_line = ResponseLine::new(
            HttpVersion::new(HttpVersionEnum::V1_1),
            Status::INTERNAL_SERVER_ERROR,
        );

        return ResponseMessage::new(reponse_line, request_message.header, Body::default());
    };

    let response_line = ResponseLine::new(HttpVersion::new(HttpVersionEnum::V1_1), Status::OK);

    request_message.header.content_type = ContentType::TextHtml;
    request_message.header.content_length = ContentLength::new(contents.len() as u64);

    let Ok(body) = Body::parse(
        contents.as_bytes().to_vec(),
        &request_message.header.content_type,
    ) else {
        let response_line = ResponseLine::new(
            HttpVersion::new(HttpVersionEnum::V1_1),
            Status::INTERNAL_SERVER_ERROR,
        );

        return ResponseMessage::new(response_line, request_message.header, Body::default());
    };

    ResponseMessage::new(response_line, request_message.header, body)
}
