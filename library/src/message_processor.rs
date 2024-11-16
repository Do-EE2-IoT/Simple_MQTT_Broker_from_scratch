
pub use bincode;
pub use serde;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub enum MqttMessage {
    Publish {
        topic: String,
        qos: u8,
        message: String,
    },

    Subscribe {
        topic: String,
    },
}
