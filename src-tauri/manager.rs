//! 无人机管理中心：管理 profile 读写、MQTT/RTSP 生命周期。
use crate::config::{self, AppProfiles, DroneProfile};
use crate::db;
use crate::mjpeg::MjpegFeedManager;
use crate::remote;
use std::collections::HashMap;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct DroneManager {
    app: AppHandle,
    profiles: Mutex<AppProfiles>,
    mqtt_cancellations: Mutex<HashMap<String, CancellationToken>>,
    pub mjpeg: MjpegFeedManager,
    db_cancel_token: Mutex<Option<CancellationToken>>,
    db_pool: Mutex<Option<sqlx::PgPool>>,
}

impl DroneManager {
    pub fn new(app: AppHandle) -> Self {
        let profiles = config::load_profiles(&app);
        Self {
            app,
            profiles: Mutex::new(profiles),
            mqtt_cancellations: Mutex::new(HashMap::new()),
            mjpeg: MjpegFeedManager::new(),
            db_cancel_token: Mutex::new(None),
            db_pool: Mutex::new(None),
        }
    }

    async fn save(&self) -> Result<(), String> {
        let profiles = self.profiles.lock().await;
        config::save_profiles(&self.app, &profiles)
    }

    pub async fn get_profiles(&self) -> Vec<DroneProfile> {
        self.profiles.lock().await.profiles.clone()
    }

    pub async fn save_profile(&self, profile: DroneProfile) -> Result<(), String> {
        {
            let mut profiles = self.profiles.lock().await;
            if let Some(existing) = profiles.profiles.iter_mut().find(|p| p.name == profile.name) {
                *existing = profile.clone();
            } else {
                profiles.profiles.push(profile.clone());
            }
            if profiles.active_drone_name.is_none() {
                profiles.active_drone_name = Some(profile.name.clone());
            }
        }
        self.save().await?;
        self.start_services_for(&profile).await;
        Ok(())
    }

    pub async fn delete_profile(&self, name: &str) -> Result<(), String> {
        {
            let mut profiles = self.profiles.lock().await;
            profiles.profiles.retain(|p| p.name != name);
            if profiles.active_drone_name.as_deref() == Some(name) {
                profiles.active_drone_name = profiles.profiles.first().map(|p| p.name.clone());
            }
        }
        self.save().await?;
        self.stop_services_for(name).await;
        db::LAST_SAMPLE.write().await.remove(name);
        Ok(())
    }

    pub async fn switch_active(&self, name: &str) -> Result<(), String> {
        let mut profiles = self.profiles.lock().await;
        if profiles.profiles.iter().any(|p| p.name == name) {
            profiles.active_drone_name = Some(name.to_string());
            drop(profiles);
            self.save().await
        } else {
            Err(format!("drone '{}' not found", name))
        }
    }

    pub async fn get_active_drone(&self) -> Option<String> {
        self.profiles.lock().await.active_drone_name.clone()
    }

    pub fn jpeg_path_for(&self, drone_name: &str) -> std::path::PathBuf {
        crate::mjpeg::last_jpeg_path(drone_name)
    }

    async fn start_services_for(&self, profile: &DroneProfile) {
        if !profile.enabled {
            return;
        }
        self.start_mqtt_for(profile).await;
        let is_active = self.get_active_drone().await.as_deref() == Some(&profile.name);
        if is_active && !profile.rtsp_url.is_empty() {
            self.mjpeg
                .start_feed(profile.name.clone(), profile.rtsp_url.clone()).await;
        }
    }

    async fn stop_services_for(&self, name: &str) {
        {
            let mut cancellations = self.mqtt_cancellations.lock().await;
            if let Some(cancel) = cancellations.remove(name) {
                cancel.cancel();
            }
        }
        self.mjpeg.stop_feed(name).await;
    }

    async fn start_mqtt_for(&self, profile: &DroneProfile) {
        let cancel = CancellationToken::new();
        {
            self.mqtt_cancellations
                .lock()
                .await
                .insert(profile.name.clone(), cancel.clone());
        }
        let app = self.app.clone();
        let profile = profile.clone();
        let drone_name = profile.name.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = remote::run_mqtt_for_profile(app, profile) => {}
                _ = cancel.cancelled() => {
                    log::info!("mqtt[{}]: stopped by cancellation", drone_name);
                }
            }
        });
    }

    pub async fn start_all_enabled(&self) {
        let (db_url, interval, db_enabled) = {
            let profiles = self.profiles.lock().await;
            (profiles.database_url.clone(), profiles.measure_persist_interval_secs, profiles.db_enabled)
        };
        if !db_url.is_empty() && db_enabled {
            match db::init_pool(&db_url).await {
                Ok(pool) => {
                    *self.db_pool.lock().await = Some(pool.clone());
                    let cancel = CancellationToken::new();
                    *self.db_cancel_token.lock().await = Some(cancel.clone());
                    db::spawn_periodic_insert(pool, interval, cancel);
                }
                Err(e) => {
                    log::error!("db init pool: {}", e);
                }
            }
        }

        let profiles = self.profiles.lock().await.clone();
        for profile in &profiles.profiles {
            if profile.enabled {
                self.start_mqtt_for(profile).await;
            }
        }
        if let Some(ref active_name) = profiles.active_drone_name {
            if let Some(profile) = profiles.profiles.iter().find(|p| &p.name == active_name) {
                if !profile.rtsp_url.is_empty() {
                    self.mjpeg
                        .start_feed(profile.name.clone(), profile.rtsp_url.clone()).await;
                }
            }
        }
    }

    pub async fn get_global_settings(&self) -> Result<(String, u64, bool), String> {
        let profiles = self.profiles.lock().await;
        Ok((profiles.database_url.clone(), profiles.measure_persist_interval_secs, profiles.db_enabled))
    }

    pub async fn update_global_settings(
        &self,
        database_url: String,
        measure_persist_interval_secs: u64,
        db_enabled: bool,
    ) -> Result<(), String> {
        // Cancel existing DB task first
        {
            let mut cancel_lock = self.db_cancel_token.lock().await;
            if let Some(token) = cancel_lock.take() {
                token.cancel();
            }
        }

        // Update persisted settings
        {
            let mut profiles = self.profiles.lock().await;
            profiles.database_url = database_url.clone();
            profiles.measure_persist_interval_secs = measure_persist_interval_secs;
            profiles.db_enabled = db_enabled;
        }
        self.save().await?;

        // Re-init pool and restart if URL is non-empty, enabled, and interval > 0
        if !database_url.is_empty() && db_enabled && measure_persist_interval_secs > 0 {
            match db::init_pool(&database_url).await {
                Ok(pool) => {
                    *self.db_pool.lock().await = Some(pool.clone());
                    let cancel = CancellationToken::new();
                    *self.db_cancel_token.lock().await = Some(cancel.clone());
                    db::spawn_periodic_insert(pool, measure_persist_interval_secs, cancel);
                    log::info!("db settings reloaded: interval={}s", measure_persist_interval_secs);
                }
                Err(e) => {
                    log::error!("db re-init failed: {}", e);
                    return Err(format!("数据库连接失败: {}", e));
                }
            }
        } else {
            *self.db_pool.lock().await = None;
            log::info!("db stopped: empty URL, disabled, or zero interval");
        }
        Ok(())
    }
}
