use crate::types::{
    body::{Body, BodyType},
    header::{ContentLength, ContentType},
    request::RequestMessage,
    request_line::{HttpVersion, HttpVersionEnum, RequestType},
    response::ResponseMessage,
    response_line::ResponseLine,
    status::Status,
};

pub fn handle(mut request_message: RequestMessage) -> ResponseMessage {
    if request_message.request_line.request_type != RequestType::Get {
        let reponse_line =
            ResponseLine::new(HttpVersion::new(HttpVersionEnum::V1_1), Status::NOT_FOUND);

        return ResponseMessage::new(reponse_line, request_message.header, Body::default());
    }

    let response_line = ResponseLine::new(HttpVersion::new(HttpVersionEnum::V1_1), Status::OK);

    request_message.body = Body::new(BodyType::TextPlain("Hello from root endpoint!".to_owned()));
    request_message.header.content_type = ContentType::ApplicationJson;
    request_message.header.content_length =
        ContentLength::new(request_message.body.to_string().len() as u64);

    ResponseMessage::new(response_line, request_message.header, request_message.body)
}
