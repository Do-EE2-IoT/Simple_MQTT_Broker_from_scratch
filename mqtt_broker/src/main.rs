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
    clientid: usize,
) -> io::Result<()> {
    loop {
        tokio::select! {
                data = tcp.get_request() => {
                match data {
                    Ok(data_from_client) => {
                        if let Ok(data) = bincode::deserialize(&data_from_client){
                            match data {
                                MqttMessage::Subscribe {
                                    topic
                                } => {
                                    tx_broker.send(BrokerMessage::sub(clientid, &topic)).await.unwrap();
                                },MqttMessage::Publish {
                                    topic,qos,message
                                } => {
                                    tx_broker.send(BrokerMessage::pub_message(&topic,qos, &message)).await.unwrap();
                                },
                                MqttMessage::Ping => tx_broker.send(BrokerMessage::ping(clientid)).await.unwrap(),
                                MqttMessage::Disconnect => tx_broker.send(BrokerMessage::disconnect(clientid)).await.unwrap(),

                            }
                        }else {
                            println!("Can't deserialize message from client");
                        }
                    },Err(_) => {
                        println!("Client {clientid} disconnected");
                        break;
                    },
                }
            },data = rx_client.recv() => {
                match data {
                    Some(MqttMessage::Subscribe {
                        topic
                    }) => {
                            let data = bincode::serialize(&MqttMessage::Subscribe {
                            topic
                        }).unwrap();
                        if let Err(_e) = tcp.respond(data).await {
                            println!("Can't send sub ack to client {clientid}");
                        }
                    },Some(MqttMessage::Publish {
                        topic,qos,message
                    }) => {
                        let data = bincode::serialize(&MqttMessage::Publish {
                            topic,qos,message
                        }).unwrap();
                        if let Err(_e) = tcp.respond(data).await {
                            println!("Can't publish to client {clientid}");
                        }
                    },Some(MqttMessage::Ping) => {
                        let data = bincode::serialize(&MqttMessage::Ping).unwrap();
                        if let Err(_e) = tcp.respond(data).await {
                            println!("Can't send to client");
                            }
                    },
                    Some(MqttMessage::Disconnect) => todo!()
                    ,None => todo!()
                    ,
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
                tokio::spawn(async move {client_handle(tcp, rx_client, borrow_tx_broker, client_id).await});
            }

            Some(broker_message) = broker.rx_broker.recv() => {
                match broker_message {
                    BrokerMessage::Subscribe { id, topic } => {
                        if let Err(e) = broker.add_subscriber(id, topic.as_str()) {
                            eprintln!("Failed to add subscriber: {}", e);
                        }
                        if let Some(tx_client) = broker.clients.get(&id) {
                            if let Err(e) = tx_client.send(MqttMessage::sub(&topic)).await {
                                println!("Error send {e}");
                            }
                        }
                    }
                    BrokerMessage::Publish { topic, message, qos } => {
                        println!("a client publishing to topic '{}': {} - qos{}", topic, message, qos);
                        let get_clients_sub = broker.subscriber.get(topic.as_str());
                        match get_clients_sub{
                            Some(clients)=> {
                                for clientid in clients.iter(){
                                    if qos == 0 {
                                    if let Some(tx_client) = broker.clients.get(clientid){
                                          if let Err(e) =  tx_client.send(MqttMessage::pub_message(&topic, qos, &message)).await{
                                            println!("{e}");
                                          }
                                    }else if qos == 1 {
                                        todo!();
                                    }
                                    else if qos == 2{
                                        todo!();
                                    }else {
                                        println!("broker not support this qos");
                                    }
                                }
                                }
                            },
                            None=> println!("Dont have any client sub this topic"),
                        }



                    }
                    BrokerMessage::Ping { id } => {
                        println!("Get Ping from client {id}");
                         let get_tx_client = broker.clients.get(&id);
                        match get_tx_client{
                            Some(tx_client) => tx_client.send(MqttMessage::Ping).await.unwrap(),
                            None=> todo!(),
                        }

                    }
                    BrokerMessage::Disconnect { id } => {
                        println!("Client {} disconnected", id);
                        broker.clients.remove(&id);
                        todo!()
                    }
                }
            }
        }
    }
}
