use std::io;

use library::message_processor::bincode;
use library::message_processor::MqttMessage;
use library::protocol::mqtt::mqtt_broker;
use library::protocol::mqtt::mqtt_broker::{BrokerMessage, Receiver, Sender};
use library::tcp_stream_handler::server::ServerStreamHandler;
use library::tcp_stream_handler::tokio::{self};

async fn client_handle(
    mut tcp: ServerStreamHandler,
    mut rx_client: Receiver<MqttMessage>,
    tx_broker: Sender<BrokerMessage>,
) -> io::Result<()> {
    loop {
        tokio::select! {
            data =tcp.get_request() => {
                match data{
                    Ok(data_from_client) => {
                        let data: MqttMessage = bincode::deserialize(&data_from_client).unwrap();
                        println!("{:?}",data);
                        //let data = BrokerMessage::Subscribe { id: data, topic: data };
                        // tx_broker.send(&data).await.unwrap();

                    },
                    Err(e) => todo!(),
                }

            },

            data = rx_client.recv()=> {
               match data{
                    Some( MqttMessage::Subscribe { topic } )=> {
                        println!("Send topic to client");
                        let data = bincode::serialize(&MqttMessage::Subscribe { topic }).unwrap();
                        tcp.respond(data).await.unwrap();
                    },
                    Some(MqttMessage::Publish{ topic, qos, message })=>{},
                    None=> {},
               }
            }



        }
    }
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
            Ok(tcp) = ServerStreamHandler::new_socket(&listener) => {
                println!("New client with ip {:?}, clientid = {}", tcp.socket_addr, client_id);
                client_id += 1;
                let (tx_client,rx_client):(Sender<MqttMessage>, Receiver<MqttMessage>) =  tokio::sync::mpsc::channel(100);
                broker.clients.insert(client_id, tx_client.clone());
                let borrow_tx_broker = tx_broker.clone();
                tokio::spawn(async move {client_handle(tcp, rx_client, borrow_tx_broker).await});
            }

            Some(broker_message) = broker.rx_broker.recv() => {
                match broker_message {
                    BrokerMessage::Subscribe { id, topic } => {
                        if let Err(e) = broker.add_subscriber(id, topic.as_str()) {
                            eprintln!("Failed to add subscriber: {}", e);
                        }
                        if let Some(tx_client) = broker.clients.get(&id) {
                            if let Err(e) = tx_client.send(MqttMessage::Subscribe { topic: topic.to_string() }).await {
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
