use serde::{Deserialize, Serialize};

use crate::types::{
    body::{Body, BodyType},
    header::{ContentLength, ContentType},
    request::RequestMessage,
    request_line::{HttpVersion, HttpVersionEnum, RequestType},
    response::ResponseMessage,
    response_line::ResponseLine,
    status::Status,
};

#[derive(Deserialize)]
struct ApiRequest {
    a: u64,
    b: u64,
}

#[derive(Serialize)]
struct ApiResponse {
    c: u64,
}

pub fn handle(mut request_message: RequestMessage) -> ResponseMessage {
    if request_message.request_line.request_type != RequestType::Get {
        let reponse_line =
            ResponseLine::new(HttpVersion::new(HttpVersionEnum::V1_1), Status::NOT_FOUND);

        return ResponseMessage::new(reponse_line, request_message.header, Body::default());
    }

    if request_message.header.content_type != ContentType::ApplicationJson {
        let reponse_line = ResponseLine::new(
            HttpVersion::new(HttpVersionEnum::V1_1),
            Status::INTERNAL_SERVER_ERROR,
        );

        return ResponseMessage::new(reponse_line, request_message.header, Body::default());
    }

    let BodyType::ApplicationJson(value) = request_message.body.get_type().clone() else {
        let reponse_line = ResponseLine::new(
            HttpVersion::new(HttpVersionEnum::V1_1),
            Status::INTERNAL_SERVER_ERROR,
        );

        return ResponseMessage::new(reponse_line, request_message.header, Body::default());
    };

    let Ok(api_request) = serde_json::from_value::<ApiRequest>(value) else {
        let reponse_line = ResponseLine::new(
            HttpVersion::new(HttpVersionEnum::V1_1),
            Status::INTERNAL_SERVER_ERROR,
        );

        return ResponseMessage::new(reponse_line, request_message.header, Body::default());
    };

    let Ok(api_response) = serde_json::to_value(ApiResponse {
        c: api_request.a + api_request.b,
    }) else {
        let reponse_line = ResponseLine::new(
            HttpVersion::new(HttpVersionEnum::V1_1),
            Status::INTERNAL_SERVER_ERROR,
        );

        return ResponseMessage::new(reponse_line, request_message.header, Body::default());
    };

    let response_line = ResponseLine::new(HttpVersion::new(HttpVersionEnum::V1_1), Status::OK);

    request_message.body = Body::new(BodyType::ApplicationJson(api_response.clone()));
    request_message.header.content_type = ContentType::ApplicationJson;
    request_message.header.content_length =
        ContentLength::new(api_response.to_string().len() as u64);

    ResponseMessage::new(response_line, request_message.header, request_message.body)
}
