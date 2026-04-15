//! 用本机 `ffmpeg` 将 RTSP 转为 HLS 文件，由 `sse` 同端口静态目录 `/hls/` 提供，供浏览器 `hls.js` 播放。
use crate::env::CONFIG;
use std::path::PathBuf;

pub fn hls_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("var/hls")
}

/// 后台拉 RTSP、写 `var/hls/index.m3u8` + `seg*.ts`；需已安装 `ffmpeg`。
pub fn spawn_rtsp_to_hls_relay() {
    if !CONFIG.rtsp_relay_enabled {
        log::info!("RTSP relay disabled (RTSP_RELAY_ENABLED=false)");
        return;
    }
    if !CONFIG.rtsp_relay_write_hls {
        log::info!("RTSP→HLS files skipped (RTSP_RELAY_WRITE_HLS=false); use /mjpeg if needed");
        return;
    }

    let dir = hls_root();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        log::error!("create HLS dir {:?}: {}", dir, e);
        return;
    }

    let source = CONFIG.rtsp_relay_source.clone();
    let playlist = dir.join("index.m3u8");
    let segment_tpl = dir.join("seg%03d.ts");

    tokio::spawn(async move {
        log::info!(
            "starting ffmpeg RTSP→HLS relay: {} -> {}",
            source,
            playlist.display()
        );

        let mut cmd = tokio::process::Command::new("ffmpeg");
        cmd.kill_on_drop(true)
            .stdin(std::process::Stdio::null())
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("warning")
            .arg("-rtsp_transport")
            .arg("tcp")
            .arg("-i")
            .arg(&source)
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("veryfast")
            .arg("-tune")
            .arg("zerolatency")
            .arg("-c:a")
            .arg("aac")
            .arg("-f")
            .arg("hls")
            .arg("-hls_time")
            .arg("2")
            .arg("-hls_list_size")
            .arg("8")
            .arg("-hls_flags")
            .arg("delete_segments+append_list")
            .arg("-hls_segment_filename")
            .arg(segment_tpl.as_os_str())
            .arg(playlist.as_os_str());

        match cmd.spawn() {
            Ok(mut child) => {
                let status = child.wait().await;
                log::warn!("ffmpeg relay exited: {:?}", status);
            }
            Err(e) => {
                log::error!(
                    "ffmpeg not started (is `ffmpeg` installed?): {}",
                    e
                );
            }
        }
    });
}
