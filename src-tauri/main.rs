mod config;
mod data;
mod db;
mod dto;
mod manager;
mod mjpeg;
mod remote;

use crate::config::DroneProfile;
use crate::manager::DroneManager;
use serde::Serialize;
use std::sync::Arc;
use tauri::{Manager, State};

#[derive(Serialize)]
struct MjpegPath {
    last_jpeg: String,
}

#[tauri::command]
fn mjpeg_asset_path_for_drone(
    manager: State<Arc<DroneManager>>,
    drone_name: Option<String>,
) -> Result<MjpegPath, String> {
    let name = drone_name.ok_or("No active drone")?;
    let path = manager.jpeg_path_for(&name);
    Ok(MjpegPath {
        last_jpeg: path.to_string_lossy().into_owned(),
    })
}

#[derive(Serialize)]
struct GlobalSettings {
    database_url: String,
    measure_persist_interval_secs: u64,
    db_enabled: bool,
}

#[tauri::command]
async fn get_global_settings(manager: State<'_, Arc<DroneManager>>) -> Result<GlobalSettings, String> {
    let (database_url, measure_persist_interval_secs, db_enabled) = manager.get_global_settings().await?;
    Ok(GlobalSettings {
        database_url,
        measure_persist_interval_secs,
        db_enabled,
    })
}

#[tauri::command]
async fn update_global_settings(
    manager: State<'_, Arc<DroneManager>>,
    database_url: String,
    measure_persist_interval_secs: u64,
    db_enabled: bool,
) -> Result<(), String> {
    manager.update_global_settings(database_url, measure_persist_interval_secs, db_enabled).await
}

#[tauri::command]
async fn get_drone_profiles(manager: State<'_, Arc<DroneManager>>) -> Result<Vec<DroneProfile>, String> {
    Ok(manager.get_profiles().await)
}

#[tauri::command]
async fn save_drone_profile(
    manager: State<'_, Arc<DroneManager>>,
    profile: DroneProfile,
) -> Result<(), String> {
    manager.save_profile(profile).await
}

#[tauri::command]
async fn delete_drone_profile(
    manager: State<'_, Arc<DroneManager>>,
    name: String,
) -> Result<(), String> {
    manager.delete_profile(&name).await
}

#[tauri::command]
async fn switch_active_drone(
    manager: State<'_, Arc<DroneManager>>,
    name: String,
) -> Result<(), String> {
    manager.switch_active(&name).await
}

#[tauri::command]
async fn get_active_drone(manager: State<'_, Arc<DroneManager>>) -> Result<Option<String>, String> {
    Ok(manager.get_active_drone().await)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let manager = Arc::new(DroneManager::new(handle.clone()));
            app.manage(manager.clone());

            let manager_clone = manager.clone();
            tokio::spawn(async move {
                manager_clone.start_all_enabled().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            mjpeg_asset_path_for_drone,
            get_drone_profiles,
            save_drone_profile,
            delete_drone_profile,
            switch_active_drone,
            get_active_drone,
            get_global_settings,
            update_global_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
