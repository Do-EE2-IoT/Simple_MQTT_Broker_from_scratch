use crate::message_processor::MqttMessage;
use async_trait::async_trait;
use std::{io, str::FromStr};
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct ConsoleInput {
    pub buffer: String,
}

pub enum UserRequest {
    Publish,
    Subscribe,
    Disconnect,
}

#[async_trait::async_trait]
pub trait Input {
    async fn pop(&mut self) -> UserRequest;
    async fn pop_mqtt_req(&mut self) -> MqttMessage;
}

impl FromStr for UserRequest {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1" => Ok(UserRequest::Publish),
            "2" => Ok(UserRequest::Subscribe),
            "3" => Ok(UserRequest::Disconnect),
            other => Err(format!("{other} command invalid")),
        }
    }
}

impl FromStr for MqttMessage {
    type Err = String;
    // format: pub topic_name qos payload : example:  "pub /hello 0 Hello world"
    // format: sub topic_name             : example:  "sub /hello"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            Err("Empty input".to_string())
        } else {
            let command = parts[0].to_lowercase();
            match command.as_str() {
                "pub" => {
                    if parts.len() < 4 {
                        Err("Need at least 4 input: pub topic_name qos payload".to_string())
                    } else {
                        let topic = parts[1].to_string();
                        let qos = parts[2]
                            .parse::<u8>()
                            .map_err(|_| "Not a valiid number".to_string())
                            .and_then(|value| {
                                if value <= 2 {
                                    Ok(value)
                                } else {
                                    Err("Invalid QoS: must be 0, 1".to_string())
                                }
                            })?;

                        let payload = parts[3].to_string();
                        Ok(MqttMessage::pub_message(&topic, qos, &payload))
                    }
                }
                "sub" => todo!(),
                _ => Err("Command invalid".to_string()),
            }
        }
    }
}

#[async_trait::async_trait]
impl Input for ConsoleInput {
    async fn pop(&mut self) -> UserRequest {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);

        loop {
            self.buffer.clear();

            match reader.read_line(&mut self.buffer).await {
                Ok(_) => match UserRequest::from_str(self.buffer.trim()) {
                    Ok(item) => break item,
                    Err(err) => {
                        println!("Error --- {}", err);
                    }
                },
                Err(err) => {
                    println!("Error reading input: {}", err);
                }
            }
        }
    }

    async fn pop_mqtt_req(&mut self) -> MqttMessage {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);

        loop {
            self.buffer.clear();

            match reader.read_line(&mut self.buffer).await {
                Ok(_) => match MqttMessage::from_str(self.buffer.trim()) {
                    Ok(item) => break item,
                    Err(err) => {
                        println!("Error --- {}", err);
                    }
                },
                Err(err) => {
                    println!("Error reading input: {}", err);
                }
            }
        }
    }
}
