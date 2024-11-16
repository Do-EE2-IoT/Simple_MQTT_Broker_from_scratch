use std::io;

use library::protocol::mqtt::mqtt_broker;
use library::protocol::mqtt::mqtt_broker::{BrokerMessage, Receiver, Sender};
use library::tcp_stream_handler::tokio::{self};
use library::tcp_stream_handler::TcpStreamHandler;

async fn client_handle(tcp: TcpStreamHandler, rx_client: Receiver<String>) -> io::Result<()> {
    loop {}
    Ok(())
}
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
                println!("New client with ip {:?}, clientid = {}", tcp.socket_addr, client_id);
                client_id += 1;
                let (tx_client, rx_client):(Sender<String>, Receiver<String>) =  tokio::sync::mpsc::channel(100);
                broker.clients.insert(client_id, tx_client.clone());
                tokio::spawn(async move {client_handle(tcp, rx_client).await});
            }

            Some(broker_message) = broker.rx_broker.recv() => {
                match broker_message {
                    BrokerMessage::Subscribe { id, topic } => {
                        if let Err(e) = broker.add_subscriber(id, topic.as_str()) {
                            eprintln!("Failed to add subscriber: {}", e);
                        }
                        if let Some(tx_client) = broker.clients.get(&id) {
                            if let Err(e) = tx_client.send(format!("Subscribed topic {topic}")).await {
                                println!("Error send {e}");
                            }
                        }



                    }
                    BrokerMessage::Publish { topic, message, qos } => {
                        println!("Publishing to topic '{}': {} - qos{}", topic, message, qos);
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
