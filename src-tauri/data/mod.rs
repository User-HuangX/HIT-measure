//! 数据层：把 remote（MQTT）载荷转成 [`crate::dto::MeasureSample`]，
//! 再写入广播通道，由 [`crate::sse`] 推给前端。

use crate::dto::MeasureSample;
use crate::env::CONFIG;
use log::debug;
use tokio::sync::broadcast;

/// 与 `remote` 里订阅的 MQTT 主题一致，来自 `.env` 的 `MQTT_MEASURE_TOPIC`。
pub fn mqtt_measure_topic() -> String {
    CONFIG.mqtt_measure_topic.clone()
}

/// MQTT 上行格式：**一行文本** `温度,湿度,光电`（三个小数，逗号分隔，可含首尾空白）。
/// 不再使用 JSON。
pub fn parse_measure_sample(payload: &[u8]) -> Option<MeasureSample> {
    let s = std::str::from_utf8(payload).ok()?.trim();
    if s.is_empty() {
        return None;
    }
    let mut parts = s.split(',').map(str::trim).filter(|p| !p.is_empty());
    let t = parts.next()?.parse().ok()?;
    let h = parts.next()?.parse().ok()?;
    let p = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(MeasureSample {
        temperature: t,
        humidity: h,
        photoelectric: p,
    })
}

pub fn ingest_mqtt_and_broadcast(tx: &broadcast::Sender<MeasureSample>, payload: &[u8]) {
    let Some(sample) = parse_measure_sample(payload) else {
        debug!("drop mqtt payload: need CSV `t,h,p`");
        return;
    };
    if tx.send(sample).is_err() {
        debug!("sample not broadcast (no active receivers)");
    }
}
