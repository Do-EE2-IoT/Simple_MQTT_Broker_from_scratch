use std::net::SocketAddr;
pub use tokio;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub struct TcpStreamHandler {
    pub socket: TcpStream,
    pub socket_addr: SocketAddr,
}

impl TcpStreamHandler {
    pub async fn new_socket(listener: &TcpListener) -> io::Result<TcpStreamHandler> {
        let (socket, socket_addr) = listener.accept().await.unwrap();
        Ok(TcpStreamHandler {
            socket,
            socket_addr,
        })
    }

    pub fn get_socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }
    pub async fn get_request(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = vec![1; 100];
        let size = self.socket.read(&mut buffer).await?;
        buffer.truncate(size);
        Ok(buffer.to_vec())
    }

    pub async fn respond(&mut self, data: Vec<u8>) -> io::Result<()> {
        self.socket.write_all(&data).await?;
        Ok(())
    }
}
