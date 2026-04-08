mod remote;

#[tokio::main]
async fn main(){
    // 初始化日志记录器
    pretty_env_logger::init();

    // 创建 MQTT 客户端和连接，并启动新线程进行消息发布
    tokio::spawn(remote::init_mqtt());
    tokio::spawn(remote::init_rtsp());

    // 启动后端
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![

        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
