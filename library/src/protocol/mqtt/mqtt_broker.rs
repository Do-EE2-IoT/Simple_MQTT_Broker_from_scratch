use crate::message_processor::MqttMessage;
use crate::tcp_stream_handler::server::ServerStreamHandler;
use std::{collections::HashMap, io};
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
        id:usize
    },
    Ping {
        id: usize,
    },
    Disconnect {
        id: usize,
    },
}

impl BrokerMessage {
    pub fn pub_message(topic: &String, qos: u8, message: &String, id: usize) -> Self {
        BrokerMessage::Publish {
            topic: topic.to_string(),
            message: message.to_string(),
            qos,
            id,
        }
    }

    pub fn sub(id: usize, topic: &String, qos: u8) -> Self {
        BrokerMessage::Subscribe {
            id,
            topic: topic.to_string(),
            qos,
        }
    }

    pub fn ping(id: usize) -> Self {
        BrokerMessage::Ping { id }
    }

    pub fn disconnect(id: usize) -> Self {
        BrokerMessage::Disconnect { id }
    }
}

pub struct BrokerManager {
    pub subscriber: HashMap<String, Vec<usize>>,
    pub clients: HashMap<usize, Sender<MqttMessage>>,
    pub rx_broker: Receiver<BrokerMessage>,
}

pub struct BrokerStream {
    pub stream: ServerStreamHandler,
}

impl BrokerStream {
    pub fn new(stream: ServerStreamHandler) -> Self {
        Self { stream }
    }

    pub async fn send(&mut self, mqtt_message: &MqttMessage) -> io::Result<()> {
        let data = bincode::serialize(mqtt_message).unwrap();
        if let Err(e) = self.stream.respond(data).await {
            println!("Can't send puback for client {e}");
        }

        Ok(())
    }
}
impl BrokerManager {
    pub fn new(rx_broker: Receiver<BrokerMessage>) -> Self {
        Self {
            subscriber: HashMap::new(),
            clients: HashMap::new(),
            rx_broker,
        }
    }

    pub async fn add_subscriber(
        &mut self,
        client_id: usize,
        topic: &str,
        qos: u8,
    ) -> io::Result<()> {
        println!("Client {client_id} subscribe topic {topic} with qos {qos}");
        self.subscriber
            .entry(topic.to_string())
            .or_default()
            .push(client_id);
        if let Some(tx_client) = self.clients.get(&client_id) {
            if let Err(e) = tx_client
                .send(MqttMessage::sub(&topic.to_string(), qos))
                .await
            {
                println!("Error sending: {e}");
            }
        }
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

    pub async fn publish_to_subscriber(
        &mut self,
        topic: &str,
        qos: u8,
        message: &str,
        clientid: usize
    ) -> io::Result<()> {
        println!("Client ID {clientid} publishing to topic '{topic}': {message} - qos{qos}");
        if let Some(clients) = self.subscriber.get(topic) {
            for id in clients {
                if let Some(tx_client) = self.clients.get(id) {
                    if let Err(e) = tx_client
                        .send(MqttMessage::pub_message(
                            &topic.to_string(),
                            qos,
                            &message.to_string(),
                        ))
                        .await
                    {
                        println!("{e}");
                    }
                }
            }
        } else {
            println!("No clients subscribed to topic '{topic}'");
        }

        Ok(())
    }

    pub async fn send_pingres_to_client(&mut self, id: usize) -> io::Result<()> {
        println!("Received Ping from client {id}");
        if let Some(tx_client) = self.clients.get(&id) {
            if let Err(e) = tx_client.send(MqttMessage::Ping).await {
                println!("Can't send ping to dedicate client");
                println!("Error: {e}");
            }
        }
        Ok(())
    }

    pub async fn remove_client(&mut self, id: usize) -> io::Result<()> {
        println!("Client {id} disconnected");
        self.clients.remove(&id);
        Ok(())
    }
}
