mod data;
mod dto;
mod env;
mod mjpeg;
mod relay_hls;
mod remote;
mod sse;

use dto::MeasureSample;
use relay_hls::hls_root;
use serde::Serialize;

#[derive(Serialize)]
struct MediaAssetPaths {
    hls_index: String,
    mjpeg_last: String,
}

/// HLS 播放列表与 MJPEG 快照的绝对路径，供前端 `convertFileSrc` + Asset Protocol 使用。
#[tauri::command]
fn media_asset_paths() -> MediaAssetPaths {
    let hls_index = hls_root().join("index.m3u8");
    let mjpeg_last = mjpeg::last_jpeg_path();
    MediaAssetPaths {
        hls_index: hls_index.to_string_lossy().into_owned(),
        mjpeg_last: mjpeg_last.to_string_lossy().into_owned(),
    }
}

#[tokio::main]
async fn main() {
    env::init();
    pretty_env_logger::init();

    let (sample_tx, _) = tokio::sync::broadcast::channel::<MeasureSample>(
        env::CONFIG.sample_broadcast_capacity,
    );

    relay_hls::spawn_rtsp_to_hls_relay();
    if env::CONFIG.rtsp_relay_enabled {
        mjpeg::start_mjpeg_feed();
    }
    tokio::spawn(sse::serve(sample_tx.clone()));
    tokio::spawn(remote::init_mqtt(sample_tx));
    tokio::spawn(remote::init_rtsp());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![media_asset_paths])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
