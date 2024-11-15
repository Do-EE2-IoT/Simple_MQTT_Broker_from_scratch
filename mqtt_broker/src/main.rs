use library::protocol::mqtt::mqtt_broker;
use library::protocol::mqtt::mqtt_broker::{BrokerMessage, Receiver, Sender};
use library::tcp_stream_handler::tokio::{self};
use library::tcp_stream_handler::TcpStreamHandler;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let mut client_id = 0;
    let (tx_broker, rx_broker): (Sender<BrokerMessage>, Receiver<BrokerMessage>) =
        tokio::sync::mpsc::channel(100);
    let mut broker = mqtt_broker::Broker::new(rx_broker);

    loop {
        tokio::select! {
            Ok(tcp) = TcpStreamHandler::new_socket(&listener) => {
                //make new thread
                client_id += 1;
            }

            Some(broker_message) = broker.rx_broker.recv() => {
                match broker_message {
                    BrokerMessage::Subscribe { id, topic, qos } => {
                        if let Err(e) = broker.add_subscriber(id, topic.as_str()) {
                            eprintln!("Failed to add subscriber: {}", e);
                        }
                    }
                    BrokerMessage::Publish { topic, message, qos } => {
                        println!("Publishing to topic '{}': {}", topic, message);
                        todo!()
                    }
                    BrokerMessage::Ping { id } => {
                        println!("Ping received from client {}", id);
                        todo!()
                    }
                    BrokerMessage::Disconnect { id } => {
                        println!("Client {} disconnected", id);
                        todo!()
                    }
                }
            }
        }
    }
}
