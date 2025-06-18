use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{request, types};

#[tracing::instrument(name = "handle")]
pub async fn handle(
    mut stream: TcpStream,
) -> Result<types::response::ResponseMessage, request::RequestMessageError> {
    let request_message = request::parse_request(&mut stream).await?;

    tracing::info!("Parsed request message: {:?}", request_message);

    let response = match request_message.request_line.uri.get_path() {
        "/" => crate::endpoints::root::handle(request_message),
        "/api" => crate::endpoints::api::handle(request_message),
        "/about" => crate::endpoints::about::handle(request_message).await,
        route => return Err(request::RequestMessageError::UnknownRoute(route.to_owned())),
    };

    stream.write_all(response.to_string().as_bytes()).await?;
    stream.flush().await?;

    Ok(response)
}
