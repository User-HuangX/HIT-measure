//! 本地 HTTP SSE：将 [`crate::dto::MeasureSample`] 以 CSV 文本 `温度,湿度,光电` 推送（与 MQTT 一致）。
use axum::{
    extract::State,
    http::Method,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use crate::dto::MeasureSample;
use crate::env::CONFIG;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct SseState {
    pub tx: broadcast::Sender<MeasureSample>,
}

fn sample_to_csv(s: &MeasureSample) -> String {
    format!(
        "{},{},{}",
        s.temperature, s.humidity, s.photoelectric
    )
}

async fn events(State(state): State<SseState>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|item| match item {
        Ok(sample) => Some(Ok(Event::default().data(sample_to_csv(&sample)))),
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

    let app = Router::new()
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
