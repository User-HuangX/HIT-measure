//! MQTT 载荷解析为 [`crate::dto::MeasureSample`]，写 [`crate::db::LAST_SAMPLE`]，再 `emit` 到前端。
use crate::db;
use crate::dto::{MeasureEvent, MeasureSample};
use log::{debug, info, warn};
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
        drone_name: String::new(),
        temperature: t,
        humidity: h,
        photoelectric: p,
    })
}

pub async fn emit_measure_from_mqtt(app: &AppHandle, drone_name: &str, payload: &[u8]) {
    let Some(mut sample) = parse_measure_sample(payload) else {
        let raw = String::from_utf8_lossy(payload);
        warn!("drop mqtt payload: need CSV `t,h,p`, got={:?}", raw);
        return;
    };
    sample.drone_name = drone_name.to_string();
    debug!("accept mqtt payload [{}]: {:?}", drone_name, sample);
    info!(
        "parsed sample [{}]: T={}°C H={}% P={}",
        drone_name, sample.temperature, sample.humidity, sample.photoelectric
    );
    db::LAST_SAMPLE
        .write()
        .await
        .insert(drone_name.to_string(), sample.clone());
    let _ = app.emit(
        "measure",
        MeasureEvent {
            drone_name: drone_name.to_string(),
            sample,
        },
    );
}
