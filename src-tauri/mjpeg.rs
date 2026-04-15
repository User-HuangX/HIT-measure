//! RTSP → 共享 JPEG 缓冲，供无 MSE 的 WebView 通过 `GET /mjpeg/last.jpg` 定时刷新预览。
//! WebKitGTK 对 `multipart/x-mixed-replace` 的 `<img>` 支持不可靠，故不用长连接 multipart。

use crate::env::CONFIG;
use axum::body::Body;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::io::{self, ErrorKind};
use std::sync::Once;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

fn find_jpeg_soi(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == [0xFF, 0xD8])
}

fn jpeg_len_from_soi(buf: &[u8]) -> Option<usize> {
    if buf.len() < 4 {
        return None;
    }
    if buf[0] != 0xFF || buf[1] != 0xD8 {
        return None;
    }
    let rel = buf[2..].windows(2).position(|w| w == [0xFF, 0xD9])?;
    Some(2 + rel + 2)
}

async fn read_next_jpeg<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut R,
    buf: &mut Vec<u8>,
) -> io::Result<Option<Vec<u8>>> {
    const CHUNK: usize = 32768;
    const MAX_BUF: usize = 8 * 1024 * 1024;
    let mut tmp = [0u8; CHUNK];

    loop {
        if buf.len() > MAX_BUF {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "MJPEG sync lost (buffer too large)",
            ));
        }

        if let Some(off) = find_jpeg_soi(buf) {
            if off > 0 {
                buf.drain(..off);
            }
            if let Some(len) = jpeg_len_from_soi(buf) {
                if buf.len() >= len {
                    let frame = buf[..len].to_vec();
                    buf.drain(..len);
                    return Ok(Some(frame));
                }
            }
        } else if buf.len() > 65536 {
            let keep = buf.len().saturating_sub(1);
            buf.drain(..keep);
        }

        let n = reader.read(&mut tmp).await?;
        if n == 0 {
            return Ok(None);
        }
        buf.extend_from_slice(&tmp[..n]);
    }
}

static LATEST_JPEG: Lazy<RwLock<Vec<u8>>> = Lazy::new(|| RwLock::new(Vec::new()));
static FEED_ONCE: Once = Once::new();

fn start_mjpeg_feed() {
    FEED_ONCE.call_once(|| {
        tokio::spawn(mjpeg_ffmpeg_loop());
    });
}

async fn mjpeg_ffmpeg_loop() {
    if !CONFIG.rtsp_relay_enabled {
        return;
    }
    let source = CONFIG.rtsp_relay_source.clone();
    loop {
        let mut cmd = Command::new("ffmpeg");
        cmd.kill_on_drop(true)
            .stdin(std::process::Stdio::null())
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("warning")
            .arg("-rtsp_transport")
            .arg("tcp")
            .arg("-i")
            .arg(&source)
            .arg("-an")
            .arg("-vf")
            .arg("fps=12")
            .arg("-f")
            .arg("image2pipe")
            .arg("-codec:v")
            .arg("mjpeg")
            .arg("-q:v")
            .arg("5")
            .arg("-");

        let mut child = match cmd.stdout(std::process::Stdio::piped()).spawn() {
            Ok(c) => c,
            Err(e) => {
                log::error!("mjpeg ffmpeg spawn: {} (is ffmpeg installed?)", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let mut stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        let mut buf = Vec::new();
        loop {
            match read_next_jpeg(&mut stdout, &mut buf).await {
                Ok(Some(frame)) => {
                    let mut w = LATEST_JPEG.write().await;
                    *w = frame;
                }
                Ok(None) => {
                    log::warn!("mjpeg ffmpeg stdout closed; restarting");
                    break;
                }
                Err(e) => {
                    log::warn!("mjpeg frame read: {}", e);
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

pub async fn mjpeg_last_jpeg() -> impl IntoResponse {
    if !CONFIG.rtsp_relay_enabled {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "RTSP relay disabled (set RTSP_RELAY_ENABLED=true)",
        )
            .into_response();
    }

    start_mjpeg_feed();

    let bytes = LATEST_JPEG.read().await.clone();
    if bytes.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "MJPEG warming up (wait for ffmpeg / first frame)",
        )
            .into_response();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
