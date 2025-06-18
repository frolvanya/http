use tokio::net::TcpListener;

use anyhow::{Context, Result};

mod endpoints;
mod request;
mod response;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global subscriber")?;

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_owned());

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
            match response::handle(stream).await {
                Ok(response_message) => {
                    tracing::info!("Generated response message as {response_message:?}");
                }
                Err(err) => tracing::error!("Error while handling incoming stream: {err:?}"),
            }
        });
    }
}
