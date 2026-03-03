use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::Router;
use tower_http::services::ServeDir;

use crate::build::{self, BuildConfig};

pub async fn serve(config: &BuildConfig, port: u16) -> Result<()> {
    build::build(config).context("initial build")?;

    let app = Router::new().fallback_service(ServeDir::new(&config.out_dir));
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("Serving http://localhost:{}/", port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind to port {}", port))?;

    axum::serve(listener, app).await.context("serve")?;
    Ok(())
}
