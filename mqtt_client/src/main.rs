use library::input::Input;
use library::input::{ConsoleInput, UserRequest};
use library::message_processor::bincode;
use library::message_processor::MqttMessage;
use library::tcp_stream_handler::client::ClientStreamHandler;
use library::tcp_stream_handler::tokio;
use library::tcp_stream_handler::tokio::time::{sleep, timeout, Duration};

#[tokio::main]
async fn main() {
    let mut socket = ClientStreamHandler::connect("127.0.0.1:8080")
        .await
        .unwrap();
    let mut console_input = ConsoleInput {
        buffer: String::new(),
    };

    loop {
        tokio::select! {
            input =console_input.pop() => {
                 match input{
                    UserRequest::Publish => {
                        let get_mqtt_pub_req = console_input.pop_mqtt_req().await;

                    },
                    UserRequest::Subscribe => {},
                    UserRequest::Disconnect => {
                        let disconnect_request = bincode::serialize(&MqttMessage::Disconnect).unwrap();
                        if let Err(e) = socket.send(disconnect_request).await {
                              println!("Cannot ping because of error {e}");
                        }
                        break;

                    },
                 }
            },
            _ = sleep(Duration::from_secs(10)) => {
                println!("Start send ping to mqtt broker");
                let ping_request = bincode::serialize(&MqttMessage::Ping).unwrap();
                if let Err(e) = socket.send(ping_request).await {
                      println!("Cannot ping because of error {e}");
                }

                match timeout(Duration::from_millis(100), socket.read()).await {
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


        }
    }
}
