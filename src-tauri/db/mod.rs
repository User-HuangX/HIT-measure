//! PostgreSQL：按间隔落库所有无人机的最新测量值。
use crate::dto::MeasureSample;
use chrono::Utc;
use once_cell::sync::Lazy;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tokio_util::sync::CancellationToken;

/// 各无人机最近一次解析成功的样本，供定时任务写入数据库。
pub static LAST_SAMPLE: Lazy<Arc<RwLock<HashMap<String, MeasureSample>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let url = database_url.trim();
    if url.is_empty() {
        return Err(sqlx::Error::Configuration(
            "database_url 为空：请配置 PostgreSQL 连接串".into(),
        ));
    }
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS measure_samples (
            id BIGSERIAL PRIMARY KEY,
            drone_name VARCHAR(255) NOT NULL DEFAULT '',
            temperature DOUBLE PRECISION NOT NULL,
            humidity DOUBLE PRECISION NOT NULL,
            photoelectric DOUBLE PRECISION NOT NULL,
            recorded_at TIMESTAMPTZ NOT NULL
        )"#,
    )
    .execute(&pool)
    .await?;
    let _ = sqlx::query("ALTER TABLE measure_samples ADD COLUMN IF NOT EXISTS drone_name VARCHAR(255) NOT NULL DEFAULT ''")
        .execute(&pool)
        .await;
    Ok(pool)
}

pub fn spawn_periodic_insert(pool: PgPool, interval_secs: u64, cancel: CancellationToken) {
    let mut insert_count: u64 = 0;
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(interval_secs.max(1)));
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    log::info!("db periodic insert: cancelled (hot-reload)");
                    break;
                }
                _ = tick.tick() => {
                    let samples = LAST_SAMPLE.read().await.clone();
                    if samples.is_empty() {
                        continue;
                    }
                    let ts = Utc::now();
                    for (drone_name, s) in &samples {
                        if let Err(e) = sqlx::query(
                            r#"INSERT INTO measure_samples (drone_name, temperature, humidity, photoelectric, recorded_at)
                               VALUES ($1, $2, $3, $4, $5)"#,
                        )
                        .bind(drone_name)
                        .bind(s.temperature)
                        .bind(s.humidity)
                        .bind(s.photoelectric)
                        .bind(ts)
                        .execute(&pool)
                        .await
                        {
                            log::warn!("db insert [{}]: {}", drone_name, e);
                        } else {
                            insert_count += 1;
                            if insert_count <= 3 || insert_count % 60 == 0 {
                                log::info!(
                                    "db insert #{} [{}]: T={}°C H={}% P={}",
                                    insert_count,
                                    drone_name,
                                    s.temperature,
                                    s.humidity,
                                    s.photoelectric,
                                );
                            }
                        }
                    }
                }
            }
        }
    });
}
