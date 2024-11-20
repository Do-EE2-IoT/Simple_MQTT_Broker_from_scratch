use crate::message_processor::MqttMessage;

use std::{io, str::FromStr};
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct ConsoleInput {
    pub buffer: String,
}

#[async_trait::async_trait]
pub trait Input {
    async fn pop(&mut self) -> io::Result<MqttMessage>;
}

impl FromStr for MqttMessage {
    type Err = String;
    // format: pub topic_name qos payload : example:  "pub /hello 0 Hello world"
    // format: sub topic_name             : example:  "sub /hello"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        println!("{:?}", parts);
        if parts.is_empty() {
            Err("Empty input".to_string())
        } else {
            let command = parts[0].to_lowercase();
            match command.as_str() {
                "pub" => {
                    if parts.len() < 4 {
                        return Err(
                            "Need at least 4 inputs: pub topic_name qos message".to_string()
                        );
                    }

                    let topic = parts[1].to_string();

                    // Parse QoS
                    let qos = parts[2]
                        .parse::<u8>()
                        .map_err(|_| "Not a valid number".to_string())
                        .and_then(|value| {
                            if value <= 2 {
                                Ok(value)
                            } else {
                                Err("Invalid QoS: must be 0, 1, or 2".to_string())
                            }
                        })?;

                    // Combine the rest as the message
                    let message = parts[3..].join(" ");

                    Ok(MqttMessage::pub_message(&topic, qos, &message))
                }

                "sub" => {
                    if parts.len() < 3 {
                        Err("Need at least 3 input: sub topic qos".to_string())
                    } else {
                        let topic = parts[1].to_string();
                        let qos = parts[2].parse::<u8>().unwrap();
                        Ok(MqttMessage::sub(&topic, qos))
                    }
                }
                "disconnect" => Ok(MqttMessage::Disconnect),
                _ => Err("Command invalid".to_string()),
            }
        }
    }
}

#[async_trait::async_trait]
impl Input for ConsoleInput {
    async fn pop(&mut self) -> io::Result<MqttMessage> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        loop {
            self.buffer.clear();
            match reader.read_line(&mut self.buffer).await {
                Ok(_) => match MqttMessage::from_str(self.buffer.trim_end()) {
                    Ok(item) => {
                        break Ok(item);
                    }
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
