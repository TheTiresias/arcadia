use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use axum::Router;
use notify::{RecursiveMode, Watcher};
use tower_http::services::ServeDir;

use crate::build::{self, BuildConfig};

pub async fn serve(config: &BuildConfig, port: u16) -> Result<()> {
    build::build(config).context("initial build")?;

    let watcher_config = config.clone();
    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .expect("create watcher");

        watcher
            .watch(&watcher_config.src_dir, RecursiveMode::Recursive)
            .expect("watch src dir");

        let mut last_build = Instant::now() - Duration::from_secs(1);

        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    use notify::EventKind::*;
                    match event.kind {
                        Create(_) | Modify(_) | Remove(_) => {
                            if last_build.elapsed() > Duration::from_millis(200) {
                                // drain any queued events before rebuilding
                                while rx.try_recv().is_ok() {}
                                let path = event
                                    .paths
                                    .first()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_default();
                                println!("Change detected: {}", path);
                                if let Err(e) = build::build(&watcher_config) {
                                    eprintln!("Build error: {}", e);
                                }
                                last_build = Instant::now();
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Err(e)) => eprintln!("Watch error: {}", e),
                Err(_) => break,
            }
        }
    });

    let app = Router::new().fallback_service(ServeDir::new(&config.out_dir));
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("Serving http://localhost:{}/", port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind to port {}", port))?;

    axum::serve(listener, app).await.context("serve")?;
    Ok(())
}
