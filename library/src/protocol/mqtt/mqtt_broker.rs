use std::{
    collections::{HashMap, HashSet},
    fmt::format,
    io,
};
pub use tokio::sync::mpsc::{Receiver, Sender};

pub enum BrokerMessage {
    Subscribe {
        id: usize,
        topic: String,
        qos: u8,
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

pub struct Broker {
    subscriber: HashMap<String, Vec<usize>>,
    clients: HashMap<usize, Sender<String>>,
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
        let _ = self.clients.get(&client_id).map(|tx| {
            tx.send(format!(
                "Get request from Client id {client_id} sub topic: {topic}"
            ))
        });
        Ok(())
    }
    pub fn add_client(&mut self, client_id: usize, tx_client: Sender<String>) -> io::Result<()> {
        self.clients.insert(client_id, tx_client);
        Ok(())
    }
}
