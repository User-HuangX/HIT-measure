mod data;
mod db;
mod dto;
mod env;
mod mjpeg;
mod remote;

use serde::Serialize;

#[derive(Serialize)]
struct MjpegPath {
    last_jpeg: String,
}

/// RTSP→MJPEG 快照路径，供 `convertFileSrc` + Asset Protocol。
#[tauri::command]
fn mjpeg_asset_path() -> MjpegPath {
    mjpeg::ensure_last_jpeg_placeholder();
    MjpegPath {
        last_jpeg: mjpeg::last_jpeg_path().to_string_lossy().into_owned(),
    }
}

#[tokio::main]
async fn main() {
    env::init();
    pretty_env_logger::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![mjpeg_asset_path])
        .setup(|app| {
            let handle = app.handle().clone();
            if env::CONFIG.rtsp_relay_enabled && !env::CONFIG.rtsp_relay_source.trim().is_empty() {
                mjpeg::start_mjpeg_feed();
            } else if env::CONFIG.rtsp_relay_enabled {
                log::warn!("RTSP_RELAY_ENABLED 但 RTSP_RELAY_SOURCE 为空，已跳过预览拉流");
            }
            tokio::spawn(async move {
                match db::init_pool().await {
                    Ok(pool) => {
                        db::spawn_periodic_insert(pool);
                    }
                    Err(e) => log::error!("postgres: {}", e),
                }
                remote::run_mqtt(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
