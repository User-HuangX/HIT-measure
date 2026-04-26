//! 在线 RTSP → 本机 ffmpeg → `var/stream/<drone_name>.jpg`，前端用 Asset Protocol 轮询预览。

use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

fn stream_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("var/stream")
}

pub fn last_jpeg_path(drone_name: &str) -> PathBuf {
    stream_root().join(format!("{}.jpg", drone_name))
}

/// 编译时嵌入的 2×2 灰块 JPEG；在 ffmpeg 写出首帧前写入磁盘，避免 Asset Protocol 报「文件不存在」。
const LAST_PLACEHOLDER_JPEG: &[u8] = include_bytes!("assets/last_placeholder.jpg");

pub fn ensure_last_jpeg_placeholder(drone_name: &str) {
    if let Err(e) = try_ensure_last_jpeg_placeholder(drone_name) {
        log::warn!("ensure {}.jpg placeholder: {}", drone_name, e);
    }
}

fn try_ensure_last_jpeg_placeholder(drone_name: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(stream_root())?;
    let path = last_jpeg_path(drone_name);
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

pub struct MjpegFeedManager {
    tasks: Arc<Mutex<std::collections::HashMap<String, CancellationToken>>>,
}

impl MjpegFeedManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn start_feed(&self, drone_name: String, rtsp_url: String) {
        let mut tasks = self.tasks.lock().await;
        // 如果已经在运行，先停止
        if let Some(cancel) = tasks.remove(&drone_name) {
            cancel.cancel();
        }
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let tasks_clone = self.tasks.clone();

        tasks.insert(drone_name.clone(), cancel);

        tokio::spawn(async move {
            mjpeg_ffmpeg_loop(&drone_name, &rtsp_url, &cancel_clone).await;
            // 结束后从 map 中移除
            tasks_clone.lock().await.remove(&drone_name);
        });
    }

    pub async fn stop_feed(&self, drone_name: &str) {
        let mut tasks = self.tasks.lock().await;
        if let Some(cancel) = tasks.remove(drone_name) {
            cancel.cancel();
        }
    }
}

async fn mjpeg_ffmpeg_loop(drone_name: &str, source: &str, cancel: &CancellationToken) {
    if source.is_empty() {
        log::info!("mjpeg[{}]: RTSP URL is empty, skipping", drone_name);
        return;
    }
    log::info!("mjpeg[{}]: starting RTSP relay from {}", drone_name, source);
    let path = last_jpeg_path(drone_name);
    if let Err(e) = tokio::fs::create_dir_all(stream_root()).await {
        log::error!("mjpeg[{}]: create stream dir: {}", drone_name, e);
        return;
    }

    ensure_last_jpeg_placeholder(drone_name);

    let mut frame_count: u64 = 0;
    loop {
        if cancel.is_cancelled() {
            log::info!("mjpeg[{}]: feed cancelled", drone_name);
            return;
        }
        log::info!("mjpeg[{}]: spawning ffmpeg", drone_name);
        let mut cmd = Command::new("ffmpeg");
        cmd.kill_on_drop(true)
            .stdin(std::process::Stdio::null())
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("warning")
            .arg("-rtsp_transport")
            .arg("tcp")
            .arg("-i")
            .arg(source)
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
                log::error!("mjpeg[{}]: ffmpeg spawn: {}", drone_name, e);
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                    _ = cancel.cancelled() => return,
                }
                continue;
            }
        };

        let mut stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                    _ = cancel.cancelled() => return,
                }
                continue;
            }
        };

        let mut buf = Vec::new();
        loop {
            if cancel.is_cancelled() {
                log::info!("mjpeg[{}]: feed cancelled during read", drone_name);
                return;
            }
            tokio::select! {
                result = read_next_jpeg(&mut stdout, &mut buf) => {
                    match result {
                        Ok(Some(frame)) => {
                            frame_count += 1;
                            if frame_count <= 3 {
                                log::info!("mjpeg[{}]: frame #{} received ({} bytes)", drone_name, frame_count, frame.len());
                            }
                            if let Err(e) = tokio::fs::write(&path, &frame).await {
                                log::warn!("mjpeg[{}]: write {:?}: {}", drone_name, path, e);
                            }
                        }
                        Ok(None) => {
                            log::warn!("mjpeg[{}]: ffmpeg stdout closed after {} frames; restarting", drone_name, frame_count);
                            break;
                        }
                        Err(e) => {
                            log::warn!("mjpeg[{}]: frame read: {}", drone_name, e);
                            break;
                        }
                    }
                }
                _ = cancel.cancelled() => {
                    log::info!("mjpeg[{}]: feed cancelled during read", drone_name);
                    return;
                }
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
            _ = cancel.cancelled() => return,
        }
    }
}
