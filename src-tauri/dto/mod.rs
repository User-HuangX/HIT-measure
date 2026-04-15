//! 测量数据 DTO：与 MQTT 解析结果一一对应，仅作数据传输/记录用。
use serde::{Deserialize, Serialize};

/// 单次采样：温度、湿度、光电（无量纲或按设备标定后的物理量，由业务约定）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct MeasureSample {
    pub temperature: f64,
    pub humidity: f64,
    pub photoelectric: f64,
}
