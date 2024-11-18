use library::message_processor::bincode;
use library::message_processor::MqttMessage;
use library::protocol::mqtt::mqtt_broker;
use library::protocol::mqtt::mqtt_broker::{BrokerMessage, Receiver, Sender};
use library::tcp_stream_handler::server::ServerStreamHandler;
use library::tcp_stream_handler::tokio::{self};
use std::io;

async fn client_handle(
    mut tcp: ServerStreamHandler,
    mut rx_client: Receiver<MqttMessage>,
    tx_broker: Sender<BrokerMessage>,
    clientid: usize,
) -> io::Result<()> {
    loop {
        tokio::select! {
            data = tcp.get_request() => {
                match data {
                    Ok(data_from_client) => {
                        if let Ok(data) = bincode::deserialize(&data_from_client) {
                            match data {
                                MqttMessage::Subscribe { topic } => {
                                    tx_broker.send(BrokerMessage::sub(clientid, &topic)).await.unwrap();
                                },
                                MqttMessage::Publish { topic, qos, message } => {
                                    tx_broker.send(BrokerMessage::pub_message(&topic, qos, &message)).await.unwrap();
                                    if qos == 1 {
                                        let puback = bincode::serialize(&MqttMessage::Pubackqos1).unwrap();
                                        if let Err(e) = tcp.respond(puback).await {
                                            println!("Can't send puback for client {e}");
                                        }
                                    }
                                },
                                MqttMessage::Ping => {
                                    tx_broker.send(BrokerMessage::ping(clientid)).await.unwrap();
                                },
                                MqttMessage::Disconnect => {
                                    tx_broker.send(BrokerMessage::disconnect(clientid)).await.unwrap();
                                },
                                _ => (),
                            }
                        } else {
                            break;
                        }
                    },
                    Err(_) => break,
                }
            },
            data = rx_client.recv() => {
                match data {
                    Some(MqttMessage::Subscribe { topic }) => {
                        let data = bincode::serialize(&MqttMessage::Subscribe { topic }).unwrap();
                        if let Err(_e) = tcp.respond(data).await {
                            println!("Can't send sub ack to client {clientid}");
                        }
                    },
                    Some(MqttMessage::Publish { topic, qos, message }) => {
                        let data = bincode::serialize(&MqttMessage::Publish { topic, qos, message }).unwrap();
                        if let Err(_e) = tcp.respond(data).await {
                            println!("Can't publish to client {clientid}");
                        }
                    },
                    Some(MqttMessage::Ping) => {
                        let data = bincode::serialize(&MqttMessage::Ping).unwrap();
                        if let Err(_e) = tcp.respond(data).await {
                            println!("Can't send to client");
                        }
                    },
                    Some(MqttMessage::Disconnect) => (),
                    Some(MqttMessage::Pubackqos1) => {
                        let data = bincode::serialize(&MqttMessage::Pubackqos1).unwrap();
                        if let Err(_e) = tcp.respond(data).await {
                            println!("Can't send to client");
                        }
                    },
                    None => (),
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
                let (tx_client, rx_client): (Sender<MqttMessage>, Receiver<MqttMessage>) = tokio::sync::mpsc::channel(100);
                broker.add_client(client_id, tx_client.clone()).unwrap();
                let borrow_tx_broker = tx_broker.clone();
                tokio::spawn(async move {
                    client_handle(tcp, rx_client, borrow_tx_broker, client_id).await
                });
            },
            Some(broker_message) = broker.rx_broker.recv() => {
                match broker_message {
                    BrokerMessage::Subscribe { id, topic } => {
                        if let Err(e) = broker.add_subscriber(id, &topic).await {
                            eprintln!("Failed to add subscriber: {e}");
                        }
                    },
                    BrokerMessage::Publish { topic, message, qos } => {
                        if let Err(e) = broker.publish_to_subscriber(&topic, qos, &message).await{
                              println!("Can't publish to subscriber");
                              println!("{e}");
                        }
                    },
                    BrokerMessage::Ping { id } => {
                        if let Err(e) = broker.send_pingres_to_client(id).await{
                              println!("Can't send ping");
                              println!("{e}");
                        }
                    },
                    BrokerMessage::Disconnect { id } => {
                        if let Err(e) = broker.remove_client(id).await{
                            println!("Can't remove client");
                            println!("{e}");
                      }
                    },
                }
            }
        }
    }
}
