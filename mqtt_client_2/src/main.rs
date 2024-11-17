use library::input::ConsoleInput;
use library::input::Input;
use library::message_processor::bincode;
use library::message_processor::MqttMessage;
use library::tcp_stream_handler::client::ClientStreamHandler;
use library::tcp_stream_handler::tokio;
use library::tcp_stream_handler::tokio::sync::mpsc;
use library::tcp_stream_handler::tokio::sync::mpsc::{Receiver, Sender};
use library::tcp_stream_handler::tokio::time::{sleep, timeout, Duration};

async fn console_input_handle(tx: Sender<MqttMessage>) {
    let mut console_input = ConsoleInput {
        buffer: String::new(),
    };
    while let Ok(data) = console_input.pop().await {
        if let Err(e) = tx.send(data).await {
            println!("Can't use channel because of error {e}");
        }
    }
}
#[tokio::main]
async fn main() {
    let mut socket = ClientStreamHandler::connect("127.0.0.1:8080")
        .await
        .unwrap();
    let (tx, mut rx): (Sender<MqttMessage>, Receiver<MqttMessage>) = mpsc::channel(1);
    tokio::spawn(console_input_handle(tx));
    loop {
        tokio::select! {
            Some(input) = rx.recv() => {
                 match input{
                    MqttMessage::Publish{topic, qos, message} => {
                        let data = bincode::serialize(&MqttMessage::pub_message(&topic, qos,&message)).unwrap();

                        if let Err(e) = socket.send(data).await{
                            println!("Error : {e}");
                        }
                    },
                    MqttMessage::Subscribe{topic} =>{
                        let data = bincode::serialize(&MqttMessage::sub(&topic)).unwrap();
                        if let Err(e) = socket.send(data).await{
                            println!("Error : {e}");
                        }
                    },
                    MqttMessage::Disconnect => {
                        let data = bincode::serialize(&MqttMessage::Disconnect).unwrap();
                        if let Err(e) = socket.send(data).await{
                            println!("Error : {e}");
                        }
                        break;
                    },
                    _ => println!("Invalid command"),
                 }
                },
            _ = sleep(Duration::from_secs(30)) => {
                println!("Start send ping to mqtt broker");
                let ping_request = bincode::serialize(&MqttMessage::Ping).unwrap();
                if let Err(e) = socket.send(ping_request).await {
                      println!("Cannot ping because of error {e}");
                }

                match timeout(Duration::from_millis(30), socket.read()).await {
                    Ok(Ok(data)) => {
                       let ping_respond: MqttMessage = bincode::deserialize(&data).unwrap();
                       if ping_respond == MqttMessage::Ping{
                          println!("Client also keep connect to mqtt broker");
                       }
                       else{
                          println!("Get broker another broker message");
                       }
                    },

                    Ok(Err(_)) => todo!(),
                    Err(e) => {
                        println!("Can't send ping to server with {e}");
                    }

                }
            },

            get_data = socket.read()=> {
                 if let Ok(data) = get_data{
                     let data: MqttMessage = bincode::deserialize(&data).unwrap();
                     match data{
                        MqttMessage::Publish{topic, qos, message} => {
                            println!("topic: {topic}, qos: {qos}");
                            println!("message: {message}");
                        },
                        MqttMessage::Subscribe { topic } => {
                            println!("Successfully subscribe to topic: {topic}");
                        },
                        _ => println!("Invalid message"),
                     }
                 }
            }


        }
    }
}
