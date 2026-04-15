mod data;
mod dto;
mod remote;
mod sse;

#[tokio::main]
async fn main(){
    // 初始化日志记录器
    pretty_env_logger::init();

    let (sample_tx, _) =
        tokio::sync::broadcast::channel::<dto::MeasureSample>(data::SAMPLE_CHANNEL_CAPACITY);

    tokio::spawn(sse::serve(sample_tx.clone()));
    tokio::spawn(remote::init_mqtt(sample_tx));
    tokio::spawn(remote::init_rtsp());

    // 启动后端
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![

        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
