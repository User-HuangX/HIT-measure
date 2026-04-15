pub mod mqtt_config;

use crate::data;
use crate::env::CONFIG;
use rumqttc::{AsyncClient, Event, Packet, QoS};
use std::time::Duration as StdDuration;
use tauri::AppHandle;

/// MQTT（仅订阅测量主题）→ 解析 → `emit("measure", …)`。
pub async fn run_mqtt(app: AppHandle) {
    let mqttoptions = mqtt_config::get_mqtt();
    let (client, mut connection) = AsyncClient::new(mqttoptions, 10);

    let topic = CONFIG.mqtt_measure_topic.clone();
    if let Err(e) = client.subscribe(topic.clone(), QoS::AtMostOnce).await {
        log::error!("mqtt subscribe {}: {:?}", topic, e);
        return;
    }

    loop {
        match connection.poll().await {
            Ok(notification) => {
                if let Event::Incoming(Packet::Publish(publish)) = notification {
                    data::emit_measure_from_mqtt(&app, &publish.payload).await;
                }
            }
            Err(e) => {
                log::warn!("mqtt connection: {:?}", e);
                tokio::time::sleep(StdDuration::from_secs(1)).await;
            }
        }
    }
}
