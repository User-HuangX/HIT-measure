pub mod mqtt_config;
pub mod rtsp_config;
use crate::data;
use crate::dto::MeasureSample;
use rumqttc::{AsyncClient, Event, Packet, QoS};
use std::time::Duration as StdDuration;
use tokio::sync::broadcast;

/// MQTT 上行 → data 层解析 → 广播给 SSE。
pub async fn init_mqtt(tx: broadcast::Sender<MeasureSample>) {
    let mqttoptions = mqtt_config::get_mqtt();
    let (client, mut connection) = AsyncClient::new(mqttoptions, 10);

    let topic = data::mqtt_measure_topic();
    if let Err(e) = client.subscribe(topic.clone(), QoS::AtMostOnce).await {
        log::error!("mqtt subscribe {}: {:?}", topic, e);
        return;
    }

    loop {
        match connection.poll().await {
            Ok(notification) => {
                if let Event::Incoming(Packet::Publish(publish)) = notification {
                    data::ingest_mqtt_and_broadcast(&tx, &publish.payload);
                }
            }
            Err(e) => {
                log::warn!("mqtt connection: {:?}", e);
                tokio::time::sleep(StdDuration::from_secs(1)).await;
            }
        }
    }
}

pub async fn init_rtsp() -> Result<(), String> {
    rtsp_config::manual_subscribe_rtsp().await.map_err(|e| e.to_string())?;
    Ok(())
}
