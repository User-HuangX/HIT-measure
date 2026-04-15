//! 数据层：把 remote（如 MQTT）的原始载荷转成 [`crate::dto::MeasureSample`]，
//! 再写入广播通道，由 [`crate::sse`] 主动推给前端。

use crate::dto::MeasureSample;
use log::debug;
use tokio::sync::broadcast;

/// 与 `remote` 里订阅的 MQTT 主题一致（测量 JSON 上行）。
pub const MQTT_MEASURE_TOPIC: &str = "hit/measure/sample";

/// 广播缓冲容量（慢消费端可能丢最旧事件，按需调大）。
pub const SAMPLE_CHANNEL_CAPACITY: usize = 256;

/// 解析 MQTT payload → DTO。默认期望 JSON：`{"temperature":f64,"humidity":f64,"photoelectric":f64}`。
pub fn parse_measure_sample(payload: &[u8]) -> Option<MeasureSample> {
    if payload.is_empty() {
        return None;
    }
    serde_json::from_slice::<MeasureSample>(payload).ok()
}

/// 转换并入队；无 SSE 订阅者时 `send` 会失败，属于正常情况。
pub fn ingest_mqtt_and_broadcast(tx: &broadcast::Sender<MeasureSample>, payload: &[u8]) {
    let Some(sample) = parse_measure_sample(payload) else {
        debug!("drop mqtt payload: parse failed or empty");
        return;
    };
    if tx.send(sample).is_err() {
        debug!("sample not broadcast (no active receivers)");
    }
}
