use async_trait::async_trait;
use std::{io, str::FromStr};
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct ConsoleInput {
    pub buffer: String,
}

pub enum UserRequest {
    Publish,
    Subscribe,
    Disconnect,
}

#[async_trait::async_trait]
pub trait Input {
    async fn pop(&mut self) -> UserRequest;
}

impl FromStr for UserRequest {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1" => Ok(UserRequest::Publish),
            "2" => Ok(UserRequest::Subscribe),
            "3" => Ok(UserRequest::Disconnect),
            other => Err(format!("{other} command invalid")),
        }
    }
}

#[async_trait::async_trait]
impl Input for ConsoleInput {
    async fn pop(&mut self) -> UserRequest {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);

        loop {
            self.buffer.clear();

            match reader.read_line(&mut self.buffer).await {
                Ok(_) => match UserRequest::from_str(self.buffer.trim()) {
                    Ok(item) => break item,
                    Err(err) => {
                        println!("Error --- {}", err);
                    }
                },
                Err(err) => {
                    println!("Error reading input: {}", err);
                }
            }
        }
    }
}
