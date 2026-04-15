use crate::env::CONFIG;
use rumqttc::MqttOptions;
use std::time::Duration;

pub fn get_mqtt() -> MqttOptions {
    let c = &*CONFIG;
    let mut o = MqttOptions::new(
        c.mqtt_client_id.clone(),
        c.mqtt_broker_host.clone(),
        c.mqtt_broker_port,
    );
    o.set_keep_alive(Duration::from_secs(c.mqtt_keep_alive_secs));
    o
}
