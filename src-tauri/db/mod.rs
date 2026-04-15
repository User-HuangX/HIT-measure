//! PostgreSQL：按 [`crate::env::CONFIG`] 间隔落库最近一次 MQTT 测量值。
use crate::dto::MeasureSample;
use crate::env::CONFIG;
use chrono::Utc;
use once_cell::sync::Lazy;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, MissedTickBehavior};

/// 最近一次解析成功的样本，供定时任务写入数据库。
pub static LAST_SAMPLE: Lazy<Arc<RwLock<Option<MeasureSample>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

pub async fn init_pool() -> Result<PgPool, sqlx::Error> {
    let url = CONFIG.database_url.trim();
    if url.is_empty() {
        return Err(sqlx::Error::Configuration(
            "DATABASE_URL 为空：请在 .env 中配置 PostgreSQL 连接串".into(),
        ));
    }
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS measure_samples (
            id BIGSERIAL PRIMARY KEY,
            temperature DOUBLE PRECISION NOT NULL,
            humidity DOUBLE PRECISION NOT NULL,
            photoelectric DOUBLE PRECISION NOT NULL,
            recorded_at TIMESTAMPTZ NOT NULL
        )"#,
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

pub fn spawn_periodic_insert(pool: PgPool) {
    let secs = CONFIG.measure_persist_interval_secs.max(1);
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(secs));
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            tick.tick().await;
            let sample = LAST_SAMPLE.read().await.clone();
            let Some(s) = sample else {
                continue;
            };
            let ts = Utc::now();
            if let Err(e) = sqlx::query(
                r#"INSERT INTO measure_samples (temperature, humidity, photoelectric, recorded_at)
                   VALUES ($1, $2, $3, $4)"#,
            )
            .bind(s.temperature)
            .bind(s.humidity)
            .bind(s.photoelectric)
            .bind(ts)
            .execute(&pool)
            .await
            {
                log::warn!("measure insert: {}", e);
            }
        }
    });
}
