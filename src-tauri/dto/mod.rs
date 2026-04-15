//! 测量三值（与 MQTT CSV 一致）。
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize)]
pub struct MeasureSample {
    pub temperature: f64,
    pub humidity: f64,
    pub photoelectric: f64,
}
