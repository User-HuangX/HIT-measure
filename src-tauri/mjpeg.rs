//! 在线 RTSP → 本机 ffmpeg → `var/stream/last.jpg`，前端用 Asset Protocol 轮询预览。

use crate::env::CONFIG;
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::sync::Once;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

pub fn stream_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("var/stream")
}

pub fn last_jpeg_path() -> PathBuf {
    stream_root().join("last.jpg")
}

/// 编译时嵌入的 2×2 灰块 JPEG；在 ffmpeg 写出首帧前写入磁盘，避免 Asset Protocol 报「文件不存在」。
const LAST_PLACEHOLDER_JPEG: &[u8] = include_bytes!("assets/last_placeholder.jpg");

/// 确保 `var/stream/last.jpg` 存在（尚无视频帧时为占位图）。
pub fn ensure_last_jpeg_placeholder() {
    if let Err(e) = try_ensure_last_jpeg_placeholder() {
        log::warn!("ensure last.jpg placeholder: {}", e);
    }
}

fn try_ensure_last_jpeg_placeholder() -> std::io::Result<()> {
    std::fs::create_dir_all(stream_root())?;
    let path = last_jpeg_path();
    if std::fs::metadata(&path).is_err() {
        std::fs::write(&path, LAST_PLACEHOLDER_JPEG)?;
    }
    Ok(())
}

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

static FEED_ONCE: Once = Once::new();

pub fn start_mjpeg_feed() {
    FEED_ONCE.call_once(|| {
        ensure_last_jpeg_placeholder();
        tokio::spawn(mjpeg_ffmpeg_loop());
    });
}

async fn mjpeg_ffmpeg_loop() {
    if !CONFIG.rtsp_relay_enabled {
        return;
    }
    let path = last_jpeg_path();
    if let Err(e) = tokio::fs::create_dir_all(stream_root()).await {
        log::error!("create stream dir {:?}: {}", stream_root(), e);
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
                    if let Err(e) = tokio::fs::write(&path, &frame).await {
                        log::warn!("write {:?}: {}", path, e);
                    }
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
