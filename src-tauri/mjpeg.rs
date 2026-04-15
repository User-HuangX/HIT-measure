//! RTSP → MJPEG（multipart），供无 MSE 的 WebView 用 `<img src="/mjpeg">` 预览。
//! 与 HLS 并行：HLS 仍给支持 hls.js / Safari 的环境；此处不依赖 `MediaSource`。

use crate::env::CONFIG;
use axum::body::Body;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use std::io::{self, ErrorKind};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

const MJPEG_BOUNDARY: &str = "mjpegboundary";

fn find_jpeg_soi(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == [0xFF, 0xD8])
}

/// 假定 `buf` 以 FFD8 开头，返回整帧长度（含结尾 FFD9）。
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
            // 避免垃圾数据撑爆缓冲；保留末字节以免截断 FFD8
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

pub async fn mjpeg_stream() -> impl IntoResponse {
    if !CONFIG.rtsp_relay_enabled {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "RTSP relay disabled (set RTSP_RELAY_ENABLED=true)",
        )
            .into_response();
    }

    let source = CONFIG.rtsp_relay_source.clone();
    let (tx, rx) = mpsc::channel::<Result<Bytes, std::convert::Infallible>>(2);

    tokio::spawn(async move {
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
                return;
            }
        };

        let mut stdout = match child.stdout.take() {
            Some(s) => s,
            None => return,
        };

        let mut buf = Vec::new();
        loop {
            match read_next_jpeg(&mut stdout, &mut buf).await {
                Ok(Some(frame)) => {
                    use std::fmt::Write;
                    let mut head = String::new();
                    let _ = write!(
                        &mut head,
                        "--{}\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        MJPEG_BOUNDARY,
                        frame.len()
                    );
                    let mut chunk = Vec::with_capacity(head.len() + frame.len() + 2);
                    chunk.extend_from_slice(head.as_bytes());
                    chunk.extend_from_slice(&frame);
                    chunk.extend_from_slice(b"\r\n");
                    if tx.send(Ok(Bytes::from(chunk))).await.is_err() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    log::warn!("mjpeg frame read: {}", e);
                    break;
                }
            }
        }
    });

    let stream = ReceiverStream::new(rx);
    Response::builder()
        .header(
            header::CONTENT_TYPE,
            format!("multipart/x-mixed-replace; boundary={}", MJPEG_BOUNDARY),
        )
        .header(header::CACHE_CONTROL, "no-cache, no-store")
        .body(Body::from_stream(stream))
        .expect("valid response")
}
