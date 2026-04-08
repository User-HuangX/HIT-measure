use rumqttc::{MqttOptions, LastWill, QoS};
use std::time::Duration;

const MQTT_CLIENT_ID: &str = "test-1";
const MQTT_BROKER_HOST: &str = "broker.emqx.io";
const MQTT_BROKER_PORT: u16 = 1883;
const MQTT_KEEP_ALIVE_SECS: u64 = 5;

const MQTT_WILL_TOPIC: &str = "hello/world";
const MQTT_WILL_PAYLOAD: &str = "good bye";
const MQTT_WILL_QOS: QoS = QoS::AtMostOnce;
const MQTT_WILL_RETAIN: bool = false;

pub fn get_mqtt() -> MqttOptions {
    // 设置 MQTT 连接选项和遗嘱消息
    let mut mqttoptions =
        MqttOptions::new(MQTT_CLIENT_ID, MQTT_BROKER_HOST, MQTT_BROKER_PORT);
    let will = LastWill::new(
        MQTT_WILL_TOPIC,
        MQTT_WILL_PAYLOAD,
        MQTT_WILL_QOS,
        MQTT_WILL_RETAIN,
    );
    mqttoptions
        .set_keep_alive(Duration::from_secs(MQTT_KEEP_ALIVE_SECS))
        .set_last_will(will);

    mqttoptions
}
