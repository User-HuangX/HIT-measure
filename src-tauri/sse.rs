//! 本地 HTTP SSE：将 [`crate::dto::MeasureSample`] 以 `data: <json>` 推送给前端。
use axum::{
    extract::State,
    http::Method,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use crate::dto::MeasureSample;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;
use tower_http::cors::{Any, CorsLayer};

pub const SSE_PORT: u16 = 5888;

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
        .allow_methods([Method::GET])
        .allow_headers(Any);

    let app = Router::new()
        .route("/events", get(events))
        .route("/health", get(health))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], SSE_PORT));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("SSE listening on http://127.0.0.1:{}/events", SSE_PORT);
    axum::serve(listener, app).await
}
