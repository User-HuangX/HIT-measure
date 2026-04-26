//! MQTT 载荷解析为 [`crate::dto::MeasureSample`]，写 [`crate::db::LAST_SAMPLE`]，再 `emit` 到前端。
use crate::db;
use crate::dto::MeasureSample;
use log::{debug, warn};
use tauri::{AppHandle, Emitter};

/// 一行 CSV：`温度,湿度,光电`（三个小数，逗号分隔）。
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

pub async fn emit_measure_from_mqtt(app: &AppHandle, payload: &[u8]) {
    let Some(sample) = parse_measure_sample(payload) else {
        let raw = String::from_utf8_lossy(payload);
        warn!("drop mqtt payload: need CSV `t,h,p`, got={:?}", raw);
        return;
    };
    debug!("accept mqtt payload: {:?}", sample);
    *db::LAST_SAMPLE.write().await = Some(sample);
    let _ = app.emit("measure", sample);
}
