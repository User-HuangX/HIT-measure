pub mod mqtt_config;
pub mod rtsp_config;
use rumqttc::{AsyncClient,Event, Packet};
use std::time::Duration as StdDuration;
//获取mqtt数据流等待后续解析
pub async fn init_mqtt() {

    // 创建 MQTT 客户端和连接，并启动新线程进行消息订阅
    let (_, mut connection) = AsyncClient::new(mqtt_config::get_mqtt(), 10);

    // 遍历并处理连接中的每个通知
    loop {
        match connection.poll().await {
            Ok(notification) => {
                if let Event::Incoming(Packet::Publish(publish)) = notification {
                    // 这里处理二进制数据: publish.payload
                    println!("Received payload size: {}", publish.payload.len());
                    // 将数据发送到解码线程/管道
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                tokio::time::sleep(StdDuration::from_secs(1)).await;
            }
        }
    }
}
//直接拉取rtsp流
pub async fn init_rtsp() -> Result<(),String> {
    rtsp_config::manual_subscribe_rtsp().await.map_err(|e| e.to_string())?;
    Ok(())
}
