use crate::message_processor::MqttMessage;
use std::{collections::HashMap, io};
pub use tokio::sync::mpsc::{Receiver, Sender};

pub enum BrokerMessage {
    Subscribe {
        id: usize,
        topic: String,
    },
    Publish {
        topic: String,
        message: String,
        qos: u8,
    },
    Ping {
        id: usize,
    },
    Disconnect {
        id: usize,
    },
}

impl BrokerMessage {
    pub fn pub_message(topic: &String, qos: u8, message: &String) -> Self {
        BrokerMessage::Publish {
            topic: topic.to_string(),
            message: message.to_string(),
            qos,
        }
    }

    pub fn sub(id: usize, topic: &String) -> Self {
        BrokerMessage::Subscribe {
            id,
            topic: topic.to_string(),
        }
    }

    pub fn ping(id: usize) -> Self {
        BrokerMessage::Ping { id }
    }

    pub fn disconnect(id: usize) -> Self {
        BrokerMessage::Disconnect { id }
    }
}

pub struct Broker {
    pub subscriber: HashMap<String, Vec<usize>>,
    pub clients: HashMap<usize, Sender<MqttMessage>>,
    pub rx_broker: Receiver<BrokerMessage>,
}

impl Broker {
    pub fn new(rx_broker: Receiver<BrokerMessage>) -> Self {
        Self {
            subscriber: HashMap::new(),
            clients: HashMap::new(),
            rx_broker,
        }
    }

    pub fn add_subscriber(&mut self, client_id: usize, topic: &str) -> io::Result<()> {
        self.subscriber
            .entry(topic.to_string())
            .or_default()
            .push(client_id);
        Ok(())
    }
    pub fn add_client(
        &mut self,
        client_id: usize,
        tx_client: Sender<MqttMessage>,
    ) -> io::Result<()> {
        self.clients.insert(client_id, tx_client);
        Ok(())
    }

}
