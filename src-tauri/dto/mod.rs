//! 测量数据 DTO：与 MQTT / SSE 载荷（CSV）一一对应。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MeasureSample {
    pub temperature: f64,
    pub humidity: f64,
    pub photoelectric: f64,
}
