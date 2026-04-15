//! 从仓库根目录 `.env` 加载配置（路径固定为 `CARGO_MANIFEST_DIR/.env`，不依赖当前工作目录）。
use once_cell::sync::Lazy;
use std::path::PathBuf;

pub struct AppConfig {
    pub sse_port: u16,
    pub sample_broadcast_capacity: usize,

    pub mqtt_client_id: String,
    pub mqtt_broker_host: String,
    pub mqtt_broker_port: u16,
    pub mqtt_keep_alive_secs: u64,
    pub mqtt_will_topic: String,
    pub mqtt_will_payload: String,
    pub mqtt_measure_topic: String,

    pub xiu_rtmp_port: usize,
    pub xiu_rtsp_port: usize,
    pub xiu_webrtc_port: usize,
    pub xiu_http_flv_port: usize,
    pub xiu_hls_port: usize,
    pub xiu_log_level: String,
}

fn var_u16(key: &str, default: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn var_usize_ports(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn var_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn var_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn var_string(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

impl AppConfig {
    fn from_env() -> Self {
        Self {
            sse_port: var_u16("SSE_PORT", 5888),
            sample_broadcast_capacity: var_usize("SAMPLE_BROADCAST_CAPACITY", 256),

            mqtt_client_id: var_string("MQTT_CLIENT_ID", "test-1"),
            mqtt_broker_host: var_string("MQTT_BROKER_HOST", "broker.emqx.io"),
            mqtt_broker_port: var_u16("MQTT_BROKER_PORT", 1883),
            mqtt_keep_alive_secs: var_u64("MQTT_KEEP_ALIVE_SECS", 5),
            mqtt_will_topic: var_string("MQTT_WILL_TOPIC", "hello/world"),
            mqtt_will_payload: var_string("MQTT_WILL_PAYLOAD", "good bye"),
            mqtt_measure_topic: var_string("MQTT_MEASURE_TOPIC", "hit/measure/sample"),

            xiu_rtmp_port: var_usize_ports("XIU_RTMP_PORT", 1935),
            xiu_rtsp_port: var_usize_ports("XIU_RTSP_PORT", 5544),
            xiu_webrtc_port: var_usize_ports("XIU_WEBRTC_PORT", 0),
            xiu_http_flv_port: var_usize_ports("XIU_HTTP_FLV_PORT", 8080),
            xiu_hls_port: var_usize_ports("XIU_HLS_PORT", 8081),
            xiu_log_level: var_string("XIU_LOG_LEVEL", "info"),
        }
    }
}

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    let _ = dotenvy::from_path(&env_path);
    AppConfig::from_env()
});

/// 启动时调用一次，保证 `.env` 已加载（`CONFIG` 首次访问也会加载）。
pub fn init() {
    Lazy::force(&CONFIG);
}
