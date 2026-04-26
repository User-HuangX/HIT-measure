//! 测量三值（与 MQTT CSV 一致）。
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MeasureSample {
    pub drone_name: String,
    pub temperature: f64,
    pub humidity: f64,
    pub photoelectric: f64,
}

/// MQTT 事件包装：前端按 drone_name 过滤。
#[derive(Debug, Clone, Serialize)]
pub struct MeasureEvent {
    pub drone_name: String,
    pub sample: MeasureSample,
}
