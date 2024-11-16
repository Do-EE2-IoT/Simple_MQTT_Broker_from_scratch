use std::net::SocketAddr;
pub use tokio;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub mod server {
    use std::net::SocketAddr;
    pub use tokio;
    use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    #[derive(Debug)]
    pub struct ServerStreamHandler {
        pub socket: TcpStream,
        pub socket_addr: SocketAddr,
    }

    impl ServerStreamHandler {
        pub async fn new_socket(listener: &TcpListener) -> io::Result<ServerStreamHandler> {
            let (socket, socket_addr) = listener.accept().await.unwrap();
            Ok(ServerStreamHandler {
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
}

pub mod client {
    use std::net::SocketAddr;
    pub use tokio;
    use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    pub struct ClientStreamHandler {
        pub socket: TcpStream,
    }

    impl ClientStreamHandler {
        pub async fn connect(addr: &str) -> io::Result<ClientStreamHandler> {
            let socket = TcpStream::connect(&addr).await.unwrap();
            Ok(ClientStreamHandler { socket })
        }

        pub async fn send(&mut self, data: Vec<u8>) -> io::Result<()> {
            self.socket.write_all(&data).await?;
            Ok(())
        }

        pub async fn read(&mut self) -> io::Result<Vec<u8>>{
            let mut buf: Vec<u8> = vec![0; 256];
            let size = self.socket.read(&mut buf).await?;
            buf.truncate(size);
            Ok(buf.to_vec())
        }
    }
}
