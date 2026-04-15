mod data;
mod dto;
mod env;
mod mjpeg;
mod relay_hls;
mod remote;
mod sse;

#[tokio::main]
async fn main(){
    env::init();
    // 初始化日志记录器
    pretty_env_logger::init();

    let (sample_tx, _) = tokio::sync::broadcast::channel::<dto::MeasureSample>(
        env::CONFIG.sample_broadcast_capacity,
    );

    relay_hls::spawn_rtsp_to_hls_relay();
    tokio::spawn(sse::serve(sample_tx.clone()));
    tokio::spawn(remote::init_mqtt(sample_tx));
    tokio::spawn(remote::init_rtsp());

    // 启动后端
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
