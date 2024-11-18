pub use bincode;
pub use serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MqttMessage {
    Publish {
        topic: String,
        qos: u8,
        message: String,
    },

    Subscribe {
        topic: String,
    },
    Ping,
    Disconnect,
    Pubackqos1,
}

impl MqttMessage {
    pub fn pub_message(topic: &String, qos: u8, message: &String) -> Self {
        MqttMessage::Publish {
            topic: topic.to_string(),
            qos,
            message: message.to_string(),
        }
    }

    pub fn sub(topic: &String) -> Self {
        MqttMessage::Subscribe {
            topic: topic.to_string(),
        }
    }
    
}
