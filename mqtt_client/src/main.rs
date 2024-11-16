use library::message_processor::bincode;
use library::message_processor::MqttMessage;
use library::tcp_stream_handler::client::ClientStreamHandler;
use library::tcp_stream_handler::tokio;
use library::tcp_stream_handler::tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut socket = ClientStreamHandler::connect("127.0.0.1:8080")
        .await
        .unwrap();
    let data = bincode::serialize(&MqttMessage::Subscribe {
        topic: "/hello".to_string(),
    })
    .unwrap();
    socket.send(data).await.unwrap();
    let data = socket.read().await.unwrap();
    let get_data: MqttMessage = bincode::deserialize(&data).unwrap();
    println!("Get respond from broker : {:?}", get_data);
}
