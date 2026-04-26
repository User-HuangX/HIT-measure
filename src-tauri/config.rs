//! JSON 配置文件系统：替代 `.env`，用户通过 UI 管理无人机 profile。
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneProfile {
    pub name: String,
    pub enabled: bool,
    pub mqtt_broker_host: String,
    pub mqtt_broker_port: u16,
    pub mqtt_topic: String,
    #[serde(default)]
    pub mqtt_username: Option<String>,
    #[serde(default)]
    pub mqtt_password: Option<String>,
    #[serde(default)]
    pub rtsp_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppProfiles {
    pub profiles: Vec<DroneProfile>,
    pub active_drone_name: Option<String>,
    #[serde(default = "default_interval")]
    pub measure_persist_interval_secs: u64,
    #[serde(default)]
    pub database_url: String,
    #[serde(default = "default_true")]
    pub db_enabled: bool,
}

fn default_interval() -> u64 {
    5
}

fn default_true() -> bool {
    true
}

impl AppProfiles {
    pub fn default_with_env_migration() -> Self {
        // 尝试从旧 .env 迁移出一个默认 profile
        let host = std::env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port: u16 = std::env::var("MQTT_BROKER_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1883);
        let topic = std::env::var("MQTT_MEASURE_TOPIC").unwrap_or_else(|_| "measure/data".into());
        let username = std::env::var("MQTT_USERNAME").ok();
        let password = std::env::var("MQTT_PASSWORD").ok();
        let rtsp = std::env::var("RTSP_RELAY_SOURCE").unwrap_or_default();
        let db_url = std::env::var("DATABASE_URL").unwrap_or_default();
        let interval: u64 = std::env::var("MEASURE_PERSIST_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        let active_drone_name = if host != "127.0.0.1" || !rtsp.is_empty() || !db_url.is_empty() {
            Some("default".to_string())
        } else {
            None
        };
        let mut profiles = Vec::new();
        if active_drone_name.is_some() {
            profiles.push(DroneProfile {
                name: "default".into(),
                enabled: true,
                mqtt_broker_host: host,
                mqtt_broker_port: port,
                mqtt_topic: topic,
                mqtt_username: username,
                mqtt_password: password,
                rtsp_url: rtsp,
            });
        }

        let db_enabled = !db_url.is_empty();
        Self {
            profiles,
            active_drone_name,
            measure_persist_interval_secs: interval,
            database_url: db_url,
            db_enabled,
        }
    }
}

pub fn config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("profiles.json")
}

pub fn load_profiles(app: &AppHandle) -> AppProfiles {
    let path = config_path(app);
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(profiles) => profiles,
            Err(e) => {
                log::warn!("config parse error at {:?}: {}, using defaults", path, e);
                AppProfiles::default_with_env_migration()
            }
        },
        Err(_) => {
            let defaults = AppProfiles::default_with_env_migration();
            let _ = save_profiles(app, &defaults);
            defaults
        }
    }
}

pub fn save_profiles(app: &AppHandle, profiles: &AppProfiles) -> Result<(), String> {
    let path = config_path(app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(profiles).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("write config: {}", e))?;
    Ok(())
}

