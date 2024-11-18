use std::io;

use crate::message_processor::MqttMessage;
use crate::tcp_stream_handler::client::ClientStreamHandler;
use tokio;
use tokio::time::timeout;
use tokio::time::Duration;
pub struct Client {
    socket: ClientStreamHandler,
}

impl Client {
    pub fn new(socket: ClientStreamHandler) -> Self {
        Self { socket }
    }

    pub async fn publish(&mut self, topic: &str, qos: u8, message: &str) -> io::Result<()> {
        let data = bincode::serialize(&MqttMessage::pub_message(
            &topic.to_string(),
            qos,
            &message.to_string(),
        ))
        .unwrap();

        if let Err(e) = self.socket.send(data).await {
            println!("Error : {e}");
        }

        if qos == 1 {
            let mut count_send = 0;
            loop {
                match timeout(Duration::from_millis(100), self.socket.read()).await {
                    Ok(Ok(data)) => {
                        let puback: MqttMessage = bincode::deserialize(&data).unwrap();
                        if puback == MqttMessage::Pubackqos1 {
                            println!("Get PUBACK from broker!");
                            break;
                        } else {
                            println!("Get broker another broker message");
                        }
                    }

                    Ok(Err(_)) => println!("Can't read anything"),
                    Err(_e) => {
                        count_send += 1;
                        if count_send > 5 {
                            break;
                        }
                        let data = bincode::serialize(&MqttMessage::pub_message(
                            &topic.to_string(),
                            qos,
                            &message.to_string(),
                        ))
                        .unwrap();

                        if let Err(err) = self.socket.send(data).await {
                            println!("Error : {err}");
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn subscribe(&mut self, topic: &str) -> io::Result<()> {
        let data = bincode::serialize(&MqttMessage::sub(&topic.to_string())).unwrap();
        if let Err(e) = self.socket.send(data).await {
            println!("Error : {e}");
        }
        Ok(())
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        let data = bincode::serialize(&MqttMessage::Disconnect).unwrap();
        if let Err(e) = self.socket.send(data).await {
            println!("Error : {e}");
        }
        Ok(())
    }

    pub async fn ping_broker(&mut self) -> Result<String, String> {
        println!();
        println!("Start send ping to mqtt broker");
        let ping_request = bincode::serialize(&MqttMessage::Ping).unwrap();
        if let Err(e) = self.socket.send(ping_request).await {
            println!("Cannot ping because of error {e}");
        }

        match timeout(Duration::from_millis(30), self.socket.read()).await {
            Ok(Ok(data)) => {
                let ping_respond: MqttMessage = bincode::deserialize(&data).unwrap();
                if ping_respond == MqttMessage::Ping {
                    Ok("Client also keep connect to mqtt broker".to_string())
                } else {
                    Err("Get broker another broker message".to_string())
                }
            }

            Ok(Err(_)) => Err("Can't read anything".to_string()),
            Err(e) => Err(format!("Can't send ping to server with {e}")),
        }
    }

    pub async fn read_broker_message(&mut self) {
        if let Ok(data) = self.socket.read().await {
            let data: MqttMessage = bincode::deserialize(&data).unwrap();
            match data {
                MqttMessage::Publish {
                    topic,
                    qos,
                    message,
                } => {
                    println!("topic: {topic}, qos: {qos}");
                    println!("message: {message}");
                }
                MqttMessage::Subscribe { topic } => {
                    println!("Successfully subscribe to topic: {topic}");
                }
                _ => println!("Invalid message"),
            }
        }
    }
}
