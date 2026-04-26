//! 多无人机 MQTT 订阅：每个 profile 独立连接，解析后 emit。
use crate::config::DroneProfile;
use crate::data;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration as StdDuration;
use tauri::AppHandle;

pub async fn run_mqtt_for_profile(app: AppHandle, profile: DroneProfile) {
    let client_id = format!("measure-{}", profile.name);
    let mut mqttoptions = MqttOptions::new(
        &client_id,
        &profile.mqtt_broker_host,
        profile.mqtt_broker_port,
    );
    mqttoptions.set_keep_alive(StdDuration::from_secs(5));
    if let (Some(u), Some(p)) = (&profile.mqtt_username, &profile.mqtt_password) {
        mqttoptions.set_credentials(u, p);
    }

    let (client, mut connection) = AsyncClient::new(mqttoptions, 10);
    let topic = profile.mqtt_topic.clone();
    let drone_name = profile.name.clone();

    log::info!(
        "mqtt[{}]: connecting to {}:{}",
        drone_name,
        profile.mqtt_broker_host,
        profile.mqtt_broker_port
    );

    let mut publish_count: u64 = 0;
    loop {
        match connection.poll().await {
            Ok(notification) => {
                if let Event::Incoming(Packet::ConnAck(_)) = notification {
                    log::info!("mqtt[{}]: connected, subscribing to {}", drone_name, topic);
                    if let Err(e) = client.subscribe(&topic, QoS::AtMostOnce).await {
                        log::error!("mqtt[{}] subscribe {}: {:?}", drone_name, topic, e);
                    } else {
                        log::info!("mqtt[{}] subscribed topic={}", drone_name, topic);
                    }
                }
                if let Event::Incoming(Packet::Publish(publish)) = notification {
                    publish_count += 1;
                    log::info!(
                        "mqtt[{}] publish #{} received on topic={}, payload={:?} bytes",
                        drone_name,
                        publish_count,
                        publish.topic,
                        publish.payload.len()
                    );
                    data::emit_measure_from_mqtt(&app, &drone_name, &publish.payload).await;
                }
            }
            Err(e) => {
                log::warn!(
                    "mqtt[{}]: error {:?}, reconnecting... (total disconnects: {})",
                    drone_name,
                    e,
                    publish_count
                );
                tokio::time::sleep(StdDuration::from_secs(1)).await;
            }
        }
    }
}
