use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::State;
use axum::http::Response;
use axum::middleware;
use axum::response::sse::{Event, Sse};
use axum::routing::get;
use axum::Router;
use futures_util::{Future, Stream};
use http_body_util::BodyExt as _;
use notify::{RecursiveMode, Watcher};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

use crate::build::{self, BuildConfig};

const RELOAD_SCRIPT: &str = concat!(
    "<script>\n",
    "(function() {\n",
    "  const es = new EventSource('/_reload');\n",
    "  es.onmessage = function() { location.reload(); };\n",
    "})();\n",
    "</script>\n"
);

/// Shared state threaded through the axum router.
#[derive(Clone)]
struct AppState {
    reload_tx: Arc<broadcast::Sender<()>>,
}

pub async fn serve(config: &BuildConfig, port: u16) -> Result<()> {
    build::build(config).context("initial build")?;

    // Broadcast channel: watcher thread sends (), SSE handlers receive.
    let (reload_tx, _) = broadcast::channel::<()>(4);
    let reload_tx = Arc::new(reload_tx);
    let thread_tx = Arc::clone(&reload_tx);

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

        // Also watch embed/ if it exists, reusing the same watcher and channel.
        let embed_dir = watcher_config.project_dir.join("embed");
        if embed_dir.exists() {
            watcher
                .watch(&embed_dir, RecursiveMode::Recursive)
                .expect("watch embed dir");
        }

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
                                } else {
                                    // Notify all connected SSE clients to reload.
                                    let _ = thread_tx.send(());
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

    let state = AppState { reload_tx };

    let app = Router::new()
        .route("/_reload", get(sse_handler))
        .fallback_service(ServeDir::new(&config.out_dir))
        .layer(middleware::map_response(inject_reload_script))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("Serving http://localhost:{}/", port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind to port {}", port))?;

    axum::serve(listener, app).await.context("serve")?;
    Ok(())
}

// ── SSE ───────────────────────────────────────────────────────────────────────

/// A `Stream` that wraps a `broadcast::Receiver<()>` and yields one SSE
/// `Event` per successful receive, ignoring lag errors.
struct ReloadStream {
    rx: broadcast::Receiver<()>,
}

impl Stream for ReloadStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            let fut = self.rx.recv();
            tokio::pin!(fut);
            match fut.poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(_)) => {
                    return Poll::Ready(Some(Ok(Event::default().data("reload"))))
                }
                Poll::Ready(Err(broadcast::error::RecvError::Lagged(_))) => continue,
                Poll::Ready(Err(broadcast::error::RecvError::Closed)) => {
                    return Poll::Ready(None)
                }
            }
        }
    }
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.reload_tx.subscribe();
    Sse::new(ReloadStream { rx }).keep_alive(
        axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)),
    )
}

// ── Body injection ────────────────────────────────────────────────────────────

/// Response middleware: if the response is HTML, inject the hot-reload script
/// before `</body>`. Non-HTML responses pass through unchanged.
async fn inject_reload_script(response: Response<Body>) -> Response<Body> {
    let is_html = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("text/html"))
        .unwrap_or(false);

    if !is_html {
        return response;
    }

    let (mut parts, body) = response.into_parts();

    let data: Vec<u8> = match body.collect().await {
        Err(_) => return Response::from_parts(parts, Body::empty()),
        Ok(buf) => buf.to_bytes().into(),
    };

    let insertion = b"</body>";
    let result: Vec<u8> = if let Some(pos) = find_subsequence(&data, insertion) {
        let mut v = Vec::with_capacity(data.len() + RELOAD_SCRIPT.len());
        v.extend_from_slice(&data[..pos]);
        v.extend_from_slice(RELOAD_SCRIPT.as_bytes());
        v.extend_from_slice(&data[pos..]);
        v
    } else {
        let mut v = data;
        v.extend_from_slice(RELOAD_SCRIPT.as_bytes());
        v
    };

    parts.headers.remove("content-length");
    Response::from_parts(parts, Body::from(result))
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}
