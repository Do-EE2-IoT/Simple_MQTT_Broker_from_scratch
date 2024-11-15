use library::tcp_stream_handler::tokio;
use library::tcp_stream_handler::tokio::net::TcpStream;


#[tokio::main]
async fn main() {
    let socket = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    println!("Hello, world!");
}
