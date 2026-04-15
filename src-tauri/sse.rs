//! 本地 HTTP SSE：将 [`crate::dto::MeasureSample`] 以 `data: <json>` 推送给前端；
//! 同端口提供 `/hls/` 静态目录（ffmpeg 生成的 HLS）与 `/mjpeg/last.jpg`（无 MSE 的 WebView 轮询预览）。
use axum::{
    extract::State,
    http::Method,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use crate::dto::MeasureSample;
use crate::env::CONFIG;
use crate::relay_hls;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct SseState {
    pub tx: broadcast::Sender<MeasureSample>,
}

async fn events(State(state): State<SseState>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|item| match item {
        Ok(sample) => serde_json::to_string(&sample)
            .ok()
            .map(|json| Ok(Event::default().data(json))),
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

async fn health() -> &'static str {
    "ok"
}

pub async fn serve(tx: broadcast::Sender<MeasureSample>) -> Result<(), std::io::Error> {
    let state = SseState { tx };
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::HEAD, Method::OPTIONS])
        .allow_headers(Any);

    let hls_dir = relay_hls::hls_root();
    let app = Router::new()
        .nest_service("/hls", ServeDir::new(hls_dir))
        .route("/mjpeg/last.jpg", get(crate::mjpeg::mjpeg_last_jpeg))
        .route("/events", get(events))
        .route("/health", get(health))
        .with_state(state)
        .layer(cors);

    let port = CONFIG.sse_port;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("SSE listening on http://127.0.0.1:{}/events", port);
    axum::serve(listener, app).await
}
