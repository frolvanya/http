use tokio::net::TcpListener;

use anyhow::{Context, Result};

mod request;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global subscriber")?;

    let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_owned());
    let port = std::env::var("PORT").unwrap_or("8080".to_owned());

    let url = format!("{host}:{port}");

    let listener = TcpListener::bind(&url).await?;
    tracing::info!("Listening on {url}");

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::error!("Failed to accept connection: {err:?}");
                continue;
            }
        };

        tokio::spawn(async move {
            match request::handle(stream).await {
                Ok(request_message) => {
                    tracing::info!("Parsed request message as {request_message:?}")
                }
                Err(err) => tracing::error!("Error while handling incoming stream: {err:?}"),
            }
        });
    }
}
