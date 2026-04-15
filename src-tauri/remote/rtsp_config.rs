use crate::env::CONFIG;
use anyhow::Result;
use xiu::config::Config;
use xiu::service::Service;

/// 用的 http 形式前端直接订阅；端口由 `.env` 中 `XIU_*` 配置。
pub async fn manual_subscribe_rtsp() -> Result<()> {
    let c = &*CONFIG;
    let cfg = Config::new(
        c.xiu_rtmp_port,
        c.xiu_rtsp_port,
        c.xiu_webrtc_port,
        c.xiu_http_flv_port,
        c.xiu_hls_port,
        c.xiu_log_level.clone(),
    );
    let mut service = Service::new(cfg);
    service.run().await?;
    Ok(())
}
