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
                    UserRequest::Publish => {},
                    UserRequest::Subscribe => {},
                    UserRequest::Disconnect => {},
                 }
            },
            _ = sleep(Duration::from_secs(10)) => {
                println!("Start send ping to mqtt broker");
                let ping_request = bincode::serialize(&MqttMessage::Ping).unwrap();
                if let Err(e) = socket.send(ping_request).await {
                      println!("Cannot ping because of error {e}");
                }

                match timeout(Duration::from_secs(5), socket.read()).await {
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

// let data = bincode::serialize(&MqttMessage::Subscribe {
//     topic: "/hello".to_string(),
// })
// .unwrap();
// socket.send(data).await.unwrap();
// let data = socket.read().await.unwrap();
// let get_data: MqttMessage = bincode::deserialize(&data).unwrap();
// println!("Get respond from broker : {:?}", get_data);
// let mut counter = 0;
// loop {
//     let message_content = format!("{counter}");
//     counter += 1;
//     let data = bincode::serialize(&MqttMessage::Publish {
//         topic: "/count_now".to_string(),
//         qos: 0,
//         message: message_content,
//     })
//     .unwrap();
//     sleep(Duration::from_secs(2)).await;

//     socket.send(data).await.unwrap();
// }
