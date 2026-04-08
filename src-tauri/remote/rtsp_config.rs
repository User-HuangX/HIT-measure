use xiu::service::Service;
use xiu::config::Config;
use anyhow::Result;
//用的http的形式前端直接订阅
pub async fn manual_subscribe_rtsp()-> Result<()>{
 // 你可以自己构造 Config，或用 config::load(...) 读取 toml/json
    let cfg = Config::new(
        1935, // rtmp
        5544, // rtsp
        0,    // webrtc
        8080, // httpflv
        8081, // hls
        "info".to_string(),
    );
    let mut service = Service::new(cfg);
    service.run().await?;
    Ok(())
}
