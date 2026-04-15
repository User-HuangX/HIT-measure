use crate::env::CONFIG;
use rumqttc::{LastWill, MqttOptions, QoS};
use std::time::Duration;

pub fn get_mqtt() -> MqttOptions {
    let c = &*CONFIG;
    let mut mqttoptions = MqttOptions::new(
        c.mqtt_client_id.clone(),
        c.mqtt_broker_host.clone(),
        c.mqtt_broker_port,
    );
    let will = LastWill::new(
        c.mqtt_will_topic.clone(),
        c.mqtt_will_payload.clone(),
        QoS::AtMostOnce,
        false,
    );
    mqttoptions
        .set_keep_alive(Duration::from_secs(c.mqtt_keep_alive_secs))
        .set_last_will(will);

    mqttoptions
}
