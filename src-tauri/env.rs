//! 从仓库根目录 `.env` 加载配置（`CARGO_MANIFEST_DIR/.env`）。
use once_cell::sync::Lazy;
use std::path::PathBuf;

pub struct AppConfig {
    pub mqtt_client_id: String,
    pub mqtt_broker_host: String,
    pub mqtt_broker_port: u16,
    pub mqtt_keep_alive_secs: u64,
    pub mqtt_measure_topic: String,

    pub rtsp_relay_enabled: bool,
    pub rtsp_relay_source: String,

    /// `postgresql://USER:PASSWORD@HOST:PORT/DATABASE`（见 `.env` 中 `DATABASE_URL`）。
    pub database_url: String,

    /// 将最近一次 MQTT 样本写入数据库的间隔（秒）。
    pub measure_persist_interval_secs: u64,
}

fn var_u16(key: &str, default: u16) -> u16 {
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

fn var_string(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn var_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(s) => matches!(s.to_lowercase().as_str(), "1" | "true" | "yes"),
        Err(_) => default,
    }
}

impl AppConfig {
    fn from_env() -> Self {
        Self {
            mqtt_client_id: var_string("MQTT_CLIENT_ID", "measure-1"),
            mqtt_broker_host: var_string("MQTT_BROKER_HOST", "127.0.0.1"),
            mqtt_broker_port: var_u16("MQTT_BROKER_PORT", 1883),
            mqtt_keep_alive_secs: var_u64("MQTT_KEEP_ALIVE_SECS", 5),
            mqtt_measure_topic: var_string("MQTT_MEASURE_TOPIC", "measure/data"),

            rtsp_relay_enabled: var_bool("RTSP_RELAY_ENABLED", false),
            rtsp_relay_source: var_string("RTSP_RELAY_SOURCE", ""),

            database_url: var_string("DATABASE_URL", ""),
            measure_persist_interval_secs: var_u64("MEASURE_PERSIST_INTERVAL_SECS", 5),
        }
    }
}

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    let _ = dotenvy::from_path(&env_path);
    AppConfig::from_env()
});

pub fn init() {
    Lazy::force(&CONFIG);
}
