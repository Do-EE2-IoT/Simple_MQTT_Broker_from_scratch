use library::input::ConsoleInput;
use library::input::Input;
use library::message_processor::MqttMessage;
use library::protocol::mqtt::mqtt_client::Client;
use library::tcp_stream_handler::client::ClientStreamHandler;
use library::tcp_stream_handler::tokio;
use library::tcp_stream_handler::tokio::sync::mpsc;
use library::tcp_stream_handler::tokio::sync::mpsc::{Receiver, Sender};
use library::tcp_stream_handler::tokio::time::{sleep, Duration};

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
    let socket = ClientStreamHandler::connect("127.0.0.1:8080")
        .await
        .unwrap();
    let mut client = Client::new(socket);
    let (tx, mut rx): (Sender<MqttMessage>, Receiver<MqttMessage>) = mpsc::channel(1);
    tokio::spawn(console_input_handle(tx));
    loop {
        tokio::select! {
            Some(input) = rx.recv() => {
                 match input{
                    MqttMessage::Publish{topic, qos, message} => {
                      if let Err(e) = client.publish(&topic, qos, &message).await{
                        println!("{e}");
                      }
                    },
                    MqttMessage::Subscribe{topic} =>{
                       if let Err(e) = client.subscribe(&topic).await{
                        println!("{e}");
                       }
                    },
                    MqttMessage::Disconnect => {
                       if let Err(e) = client.disconnect().await{
                        println!("{e}");
                        break;
                       }
                    },
                    _ => println!("Invalid command"),
                 }
                },
            _ = sleep(Duration::from_secs(30)) => {
                if let Err(e) = client.ping_broker().await{
                    println!("{e}");
                }
            },

            _ = client.read_broker_message() => println!("MQTT OK"),


        }
    }
}
